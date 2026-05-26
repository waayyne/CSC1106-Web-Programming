use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Loan {
    pub id: i32,
    pub user_id: i32,
    pub amount: Decimal,
    pub status: String, // "pending","approved","rejected"
    pub reason: Option<String>,
    pub created_at: NaiveDateTime,
}