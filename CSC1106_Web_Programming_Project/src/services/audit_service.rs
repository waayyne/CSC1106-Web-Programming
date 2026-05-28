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
