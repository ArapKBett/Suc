use actix_web::{App, HttpServer};
use chrono::{Duration, Utc};
use solana_client::rpc_client::RpcClient;
use solana_usdc_indexer::{indexer::index_usdc_transfers, web::get_transfers};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    // Solana RPC endpoint
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or("https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new(rpc_url);
    
    // Wallet to index
    let wallet = "7cMEhpt9y3inBNVv8fNnuaEbx7hKHZnLvR1KWKKxuDDU".to_string();
    
    // USDC mint address
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();
    
    // Time range: last 24 hours
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(24);
    
    // Index transfers
    let transfers = index_usdc_transfers(&client, &wallet, &usdc_mint, start_time, end_time)
        .await
        .expect("Failed to index transfers");
    
    // Start web server
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(transfers.clone()))
            .route("/transfers", actix_web::web::get().to(get_transfers))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}