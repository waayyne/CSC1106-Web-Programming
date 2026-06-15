use serde::Serialize;

#[derive(Serialize)]
pub struct AuditLogView {
    pub id: i32,
    pub user_id: Option<i32>,
    pub username: String,
    pub action: String,
    pub created_at: String,
}