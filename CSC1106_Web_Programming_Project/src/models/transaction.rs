use serde::Deserialize;
use chrono::NaiveDateTime;
use serde::{Serialize};
use sqlx::FromRow;
use rust_decimal::Decimal;

#[derive(Deserialize)]
pub struct TransferForm {
    pub transfer_by: String,
    pub recipient_identifier: String,
    pub amount: f64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TransactionRecord {
    pub id: i32,
    pub from_account_id: Option<i32>,
    pub to_account_id: Option<i32>,
    pub transaction_type: String,
    pub amount: Decimal,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}