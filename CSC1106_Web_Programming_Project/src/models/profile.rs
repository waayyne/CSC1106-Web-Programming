use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct ProfileRecord {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProfileView {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProfileRecord> for ProfileView {
    fn from(record: ProfileRecord) -> Self {
        Self {
            id: record.id,
            username: record.username,
            first_name: record.first_name,
            last_name: record.last_name,
            name: record.name,
            email: record.email,
            phone_number: record.phone_number,
            role: record.role,
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileForm {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}
