use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FixedDeposit {
    pub id: i32,
    pub user_id: i32,
    pub account_id: i32,
    pub principal_amount: Decimal,
    pub interest_rate: Decimal,
    pub interest_amount: Decimal,
    pub total_return: Decimal,
    pub duration_days: i32,
    pub maturity_seconds: i32,
    pub status: String,
    pub created_at: Option<NaiveDateTime>,
    pub maturity_at: NaiveDateTime,
    pub claimed_at: Option<NaiveDateTime>,
}
