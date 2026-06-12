use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize)]
pub struct CashFlowSummary {
    pub total_in: Decimal,
    pub total_out: Decimal,
    pub net_flow: Decimal,
    pub deposit_total: Decimal,
    pub withdraw_total: Decimal,
    pub transfer_out_total: Decimal,
    pub investment_out_total: Decimal,
    pub investment_return_total: Decimal,
}

#[derive(Deserialize)]

pub struct TransferForm {
    pub transfer_by: String,
    pub recipient_identifier: String,
    pub amount: Decimal,
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
    pub from_account_number: Option<String>,
    pub to_account_number: Option<String>,
    pub created_at: NaiveDateTime,
}
