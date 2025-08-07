use chrono::{DateTime, Utc, TimeZone};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use log::{info, warn, error};

use crate::models::{Transfer, TransferType};

pub async fn index_usdc_transfers(
    client: &RpcClient,
    wallet: &str,
    usdc_mint: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<Transfer>, Box<dyn std::error::Error>> {
    let wallet_pubkey = Pubkey::from_str(wallet)?;
    let usdc_mint_pubkey = Pubkey::from_str(usdc_mint)?;
    
    info!("Fetching signatures for wallet: {}", wallet);
    let signatures = client
        .get_signatures_for_address(&wallet_pubkey)
        .await
        .map_err(|e| {
            error!("Failed to get signatures: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;
    
    info!("Found {} signatures", signatures.len());
    let mut transfers = Vec::new();
    
    for sig_info in signatures {
        let signature = Signature::from_str(&sig_info.signature)?;
        let block_time = sig_info
            .block_time
            .map(|t| Utc.timestamp_opt(t, 0).single().ok_or("Invalid timestamp"))
            .transpose()
            .map_err(|e| {
                error!("Invalid block time for signature {}: {}", signature, e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        
        if let Some(tx_time) = block_time {
            if tx_time < start_time || tx_time > end_time {
                info!("Skipping signature {}: timestamp {} outside range", signature, tx_time);
                continue;
            }
            
            info!("Fetching transaction for signature: {}", signature);
            let tx = client
                .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
                .await
                .map_err(|e| {
                    error!("Failed to get transaction {}: {}", signature, e);
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
            
            if let Some(meta) = tx.transaction.meta {
                let pre_balances = meta.pre_token_balances.unwrap_or(vec![]);
                let post_balances = meta.post_token_balances.unwrap_or(vec![]);
                info!("Signature {}: Found {} pre_balances, {} post_balances", signature, pre_balances.len(), post_balances.len());
                
                for (pre, post) in pre_balances.iter().zip(post_balances.iter()) {
                    if pre.mint != usdc_mint_pubkey.to_string() || post.mint != usdc_mint_pubkey.to_string() {
                        continue;
                    }
                    
                    let pre_amount = pre.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let post_amount = post.ui_token_amount.ui_amount.unwrap_or(0.0);
                    
                    if pre_amount != post_amount {
                        let owner = pre.owner.as_ref().ok_or("Missing owner")?;
                        let owner_pubkey = Pubkey::from_str(owner)?;
                        let transfer_type = if owner_pubkey == wallet_pubkey {
                            if pre_amount > post_amount {
                                TransferType::Sent
                            } else {
                                TransferType::Received
                            }
                        } else {
                            continue;
                        };
                        
                        let amount = (post_amount - pre_amount).abs();
                        info!("Found transfer: {} USDC, type: {:?}", amount, transfer_type);
                        
                        transfers.push(Transfer {
                            date: tx_time,
                            amount,
                            transfer_type,
                            signature: signature.to_string(),
                        });
                    }
                }
            } else {
                warn!("No meta data for signature: {}", signature);
            }
        }
    }
    
    info!("Returning {} transfers", transfers.len());
    Ok(transfers)
}