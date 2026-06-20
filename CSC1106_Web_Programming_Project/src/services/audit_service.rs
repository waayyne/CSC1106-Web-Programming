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

const MAX_AUDIT_ACTION_LENGTH: usize = 500;

fn normalize_action(action: &str) -> Result<String, String> {
    let trimmed = action.trim();

    if trimmed.is_empty() {
        return Err("Audit action cannot be empty.".to_string());
    }

    if trimmed.chars().count() > MAX_AUDIT_ACTION_LENGTH {
        return Err(format!(
            "Audit action must be {} characters or fewer.",
            MAX_AUDIT_ACTION_LENGTH
        ));
    }

    if trimmed.chars().any(|c| c.is_control()) {
        return Err("Audit action contains invalid control characters.".to_string());
    }

    Ok(trimmed.to_string())
}

pub async fn log_action(pool: &DbPool, user_id: Option<i32>, action: &str) -> Result<(), String> {
    let action = normalize_action(action)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_action_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_action("  Logged in  "),
            Ok("Logged in".to_string())
        );
    }

    #[test]
    fn normalize_action_rejects_empty_string() {
        assert_eq!(
            normalize_action(""),
            Err("Audit action cannot be empty.".to_string())
        );
    }

    #[test]
    fn normalize_action_rejects_whitespace_only() {
        assert_eq!(
            normalize_action("   \t  "),
            Err("Audit action cannot be empty.".to_string())
        );
    }

    #[test]
    fn normalize_action_allows_exactly_max_length() {
        let action = "a".repeat(MAX_AUDIT_ACTION_LENGTH);
        assert_eq!(normalize_action(&action), Ok(action));
    }

    #[test]
    fn normalize_action_rejects_over_max_length() {
        let action = "a".repeat(MAX_AUDIT_ACTION_LENGTH + 1);
        assert_eq!(
            normalize_action(&action),
            Err(format!(
                "Audit action must be {} characters or fewer.",
                MAX_AUDIT_ACTION_LENGTH
            ))
        );
    }

    #[test]
    fn normalize_action_rejects_control_characters() {
        assert_eq!(
            normalize_action("Logged in\nfrom a new device"),
            Err("Audit action contains invalid control characters.".to_string())
        );
    }
}