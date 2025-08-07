use chrono::{DateTime, Utc, TimeZone};
use solana_client::nonblocking::rpc_client::RpcClient; // Use nonblocking RpcClient
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

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
    
    let signatures = client
        .get_signatures_for_address(&wallet_pubkey)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    let mut transfers = Vec::new();
    
    for sig_info in signatures {
        let signature = Signature::from_str(&sig_info.signature)?;
        let block_time = sig_info
            .block_time
            .map(|t| Utc.timestamp_opt(t, 0).single().ok_or("Invalid timestamp"))
            .transpose()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        if let Some(tx_time) = block_time {
            if tx_time < start_time || tx_time > end_time {
                continue;
            }
            
            let tx = client
                .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            
            if let Some(meta) = tx.transaction.meta {
                let pre_balances = meta.pre_token_balances.unwrap_or(vec![]);
                let post_balances = meta.post_token_balances.unwrap_or(vec![]);
                
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
                        
                        transfers.push(Transfer {
                            date: tx_time,
                            amount,
                            transfer_type,
                            signature: signature.to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(transfers)
}