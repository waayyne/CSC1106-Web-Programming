use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Loan {
    pub id: i32,
    pub user_id: i32,
    pub amount: Decimal,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: NaiveDateTime,
}
