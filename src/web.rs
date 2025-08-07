use actix_web::{web, HttpResponse};
use serde::Serialize;

use crate::models::Transfer;

#[derive(Serialize)]
struct TransferResponse {
    transfers: Vec<Transfer>,
}

pub async fn get_transfers(transfers: web::Data<Vec<Transfer>>) -> HttpResponse {
    HttpResponse::Ok().json(TransferResponse {
        transfers: transfers.get_ref().clone(),
    })
}
