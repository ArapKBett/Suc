use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

use crate::models::Transfer;

#[derive(Serialize)]
struct TransferResponse {
    transfers: Vec<Transfer>,
}

pub async fn get_transfers(transfers: web::Data<Vec<Transfer>>) -> impl Responder {
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>USDC Transfers</title>
            <style>
                table {{ border-collapse: collapse; width: 100%; }}
                th, td {{ border: 1px solid black; padding: 8px; text-align: left; }}
                th {{ background-color: #f2f2f2; }}
            </style>
        </head>
        <body>
            <h1>USDC Transfers for Wallet 7cMEhpt...xuDDU</h1>
            <table>
                <tr>
                    <th>Date</th>
                    <th>Amount (USDC)</th>
                    <th>Type</th>
                    <th>Signature</th>
                </tr>
                {}
            </table>
        </body>
        </html>
        "#,
        transfers
            .iter()
            .map(|t| format!(
                "<tr><td>{}</td><td>{:.6}</td><td>{:?}</td><td><a href=\"https://explorer.solana.com/tx/{}\">{}</a></td></tr>",
                t.date, t.amount, t.transfer_type, t.signature, t.signature
            ))
            .collect::<Vec<_>>()
            .join("")
    );
    HttpResponse::Ok().content_type("text/html").body(html)
}