use chrono::{DateTime, Utc};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
};
use spl_token::instruction::TokenInstruction;
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
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    let mut transfers = Vec::new();
    
    for sig_info in signatures {
        let signature = Signature::from_str(&sig_info.signature)?;
        let block_time = sig_info
            .block_time
            .map(|t| DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(t, 0)));
        
        if let Some(tx_time) = block_time {
            if tx_time < start_time || tx_time > end_time {
                continue;
            }
            
            let tx = client
                .get_transaction(&signature, solana_sdk::commitment_config::CommitmentConfig::confirmed())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            
            let transaction = tx.transaction.transaction;
            if let Some(meta) = tx.transaction.meta {
                let inner_instructions = meta.inner_instructions.unwrap_or_default();
                
                for instr in transaction.message.instructions.iter().chain(
                    inner_instructions
                        .iter()
                        .flat_map(|i| i.instructions.iter()),
                ) {
                    if let Ok(token_instr) = spl_token::instruction::decode(&instr.data) {
                        if let TokenInstruction::Transfer { amount } = token_instr {
                            let accounts = instr.accounts;
                            if accounts.len() < 3 {
                                continue;
                            }
                            
                            let source = transaction.message.account_keys[accounts[0] as usize];
                            let destination = transaction.message.account_keys[accounts[1] as usize];
                            let mint = transaction.message.account_keys[accounts[2] as usize];
                            
                            if mint != usdc_mint_pubkey {
                                continue;
                            }
                            
                            let transfer_type = if source == wallet_pubkey {
                                TransferType::Sent
                            } else if destination == wallet_pubkey {
                                TransferType::Received
                            } else {
                                continue;
                            };
                            
                            transfers.push(Transfer {
                                date: tx_time,
                                amount: amount as f64 / 1_000_000.0, // USDC has 6 decimals
                                transfer_type,
                                signature: signature.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    Ok(transfers)
}