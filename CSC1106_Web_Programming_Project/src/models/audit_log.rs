use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i32,
    pub user_id: Option<i32>,
    pub action: String,
    pub created_at: NaiveDateTime,
}