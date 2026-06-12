use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RiskInvestmentForm {
    pub amount: Decimal,
    pub risk_level: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RiskInvestment {
    pub id: i32,
    pub user_id: i32,
    pub account_id: i32,
    pub amount: Decimal,
    pub risk_level: String,
    pub result: String,
    pub return_amount: Decimal,
    pub profit_loss: Decimal,
    pub created_at: NaiveDateTime,
}