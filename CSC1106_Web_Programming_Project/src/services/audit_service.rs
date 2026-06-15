use serde::Serialize;

#[derive(Serialize)]
pub struct AuditLogView {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: String,
    pub action: String,
    pub created_at: String,
}

use crate::db::DbPool;

pub async fn log_action(pool: &DbPool, user_id: Option<i32>, action: &str) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO audit_logs (user_id, action) VALUES ($1, $2)",
        user_id,
        action
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
