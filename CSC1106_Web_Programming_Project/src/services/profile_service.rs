use sqlx::Row;

use crate::db::DbPool;
use crate::models::profile::{ChangePasswordForm, ProfileRecord, ProfileView, UpdateProfileForm};
use crate::services::auth_service;

#[derive(Debug)]
pub enum ProfileError {
    NotFound,
    PasswordTooShort,
    PasswordMismatch,
    CurrentPasswordInvalid,
    Validation,
    Database,
}

impl ProfileError {
    pub fn code(&self) -> &'static str {
        match self {
            ProfileError::NotFound => "profile_not_found",
            ProfileError::PasswordTooShort => "password_too_short",
            ProfileError::PasswordMismatch => "password_mismatch",
            ProfileError::CurrentPasswordInvalid => "current_password_invalid",
            ProfileError::Validation => "validation_error",
            ProfileError::Database => "database_error",
        }
    }
}

pub async fn get_profile(pool: &DbPool, user_id: i32) -> Result<ProfileView, sqlx::Error> {
    let record = sqlx::query_as::<_, ProfileRecord>(
        "select id, username, first_name, last_name, name, email, phone_number, role, created_at, updated_at
         from users
         where id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(ProfileView::from(record))
}

pub async fn update_profile(
    pool: &DbPool,
    user_id: i32,
    form: UpdateProfileForm,
) -> Result<(), ProfileError> {
    let first_name = form.first_name.trim();
    let last_name = form.last_name.trim();
    let email = form.email.trim();
    let phone_number = form.phone_number.trim();

    if first_name.is_empty() || last_name.is_empty() || email.is_empty() || phone_number.is_empty() {
        return Err(ProfileError::Validation);
    }

    let full_name = format!("{} {}", first_name, last_name);

    let mut tx = pool.begin().await.map_err(|_| ProfileError::Database)?;

    let result = sqlx::query(
        "update users
         set first_name = $1,
             last_name = $2,
             name = $3,
             email = $4,
             phone_number = $5,
             updated_at = current_timestamp
         where id = $6"
    )
    .bind(first_name)
    .bind(last_name)
    .bind(&full_name)
    .bind(email)
    .bind(phone_number)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ProfileError::Database)?;

    if result.rows_affected() == 0 {
        let _ = tx.rollback().await;
        return Err(ProfileError::NotFound);
    }

    if sqlx::query("insert into audit_logs (user_id, action) values ($1, $2)")
        .bind(user_id)
        .bind("Profile updated")
        .execute(&mut *tx)
        .await
        .is_err()
    {
        let _ = tx.rollback().await;
        return Err(ProfileError::Database);
    }

    tx.commit().await.map_err(|_| ProfileError::Database)?;

    Ok(())
}

pub async fn change_password(
    pool: &DbPool,
    user_id: i32,
    form: ChangePasswordForm,
) -> Result<(), ProfileError> {
    let current_password = form.current_password;
    let new_password = form.new_password;
    let confirm_password = form.confirm_password;

    if new_password.len() < 6 {
        return Err(ProfileError::PasswordTooShort);
    }

    if new_password != confirm_password {
        return Err(ProfileError::PasswordMismatch);
    }

    let row = sqlx::query("select password_hash from users where id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ProfileError::Database)?;

    let row = match row {
        Some(row) => row,
        None => return Err(ProfileError::NotFound),
    };

    let stored_password_hash: String = row.get("password_hash");

    if !auth_service::verify_password(&stored_password_hash, &current_password)
        .map_err(|_| ProfileError::Database)?
    {
        return Err(ProfileError::CurrentPasswordInvalid);
    }

    let new_password_hash = auth_service::hash_password(&new_password)
        .map_err(|_| ProfileError::Database)?;

    let mut tx = pool.begin().await.map_err(|_| ProfileError::Database)?;

    let result = sqlx::query(
        "update users
         set password_hash = $1,
             updated_at = current_timestamp
         where id = $2"
    )
    .bind(&new_password_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ProfileError::Database)?;

    if result.rows_affected() == 0 {
        let _ = tx.rollback().await;
        return Err(ProfileError::NotFound);
    }

    if sqlx::query("insert into audit_logs (user_id, action) values ($1, $2)")
        .bind(user_id)
        .bind("Password changed")
        .execute(&mut *tx)
        .await
        .is_err()
    {
        let _ = tx.rollback().await;
        return Err(ProfileError::Database);
    }

    tx.commit().await.map_err(|_| ProfileError::Database)?;

    Ok(())
}
