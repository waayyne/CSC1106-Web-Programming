use sqlx::Row;

use crate::db::DbPool;
use crate::models::profile::{ChangePasswordForm, ProfileRecord, ProfileView, UpdateProfileForm};
use crate::services::auth_service;

pub async fn get_profile(pool: &DbPool, user_id: i32) -> Result<ProfileView, String> {
    let record_result = sqlx::query_as::<_, ProfileRecord>(
        "select id, username, first_name, last_name, name, email, phone_number, role, created_at, updated_at
         from users where id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await;

    let record = match record_result {
        Ok(record) => record,
        Err(_) => return Err("profile_not_found".to_string()),
    };

    Ok(ProfileView::from(record))
}

pub async fn update_profile(
    pool: &DbPool,
    user_id: i32,
    form: UpdateProfileForm,
) -> Result<(), String> {
    let email = form.email.trim();
    let phone_number = form.phone_number.trim();
    // Load current profile to inspect role and existing names
    let current = match get_profile(pool, user_id).await {
        Ok(p) => p,
        Err(_) => return Err("profile_not_found".to_string()),
    };

    // Resolve name fields: use provided values or keep existing
    let first_name = match form.first_name {
        Some(s) => s.trim().to_string(),
        None => current.first_name.clone(),
    };

    let last_name = match form.last_name {
        Some(s) => s.trim().to_string(),
        None => current.last_name.clone(),
    };

    // If the user is a customer, only allow updating email and phone number.
    if current.role == "customer" {
        if email.is_empty() || phone_number.is_empty() {
            return Err("validation_error".to_string());
        }

        let update_result = sqlx::query(
            "update users set email = $1, phone_number = $2, updated_at = current_timestamp
             where id = $3",
        )
        .bind(email)
        .bind(phone_number)
        .bind(user_id)
        .execute(pool)
        .await;

        let result = match update_result {
            Ok(result) => result,
            Err(_) => return Err("database_error".to_string()),
        };

        if result.rows_affected() == 0 {
            return Err("profile_not_found".to_string());
        }
    } else {
        // Non-customer (staff/admin): allow updating first/last/name/email/phone
        if first_name.is_empty() || last_name.is_empty() || email.is_empty() || phone_number.is_empty()
        {
            return Err("validation_error".to_string());
        }

        let full_name = format!("{} {}", first_name, last_name);

        let update_result = sqlx::query(
            "update users set first_name = $1, last_name = $2, name = $3, email = $4, phone_number = $5, updated_at = current_timestamp
             where id = $6",
        )
        .bind(first_name)
        .bind(last_name)
        .bind(&full_name)
        .bind(email)
        .bind(phone_number)
        .bind(user_id)
        .execute(pool)
        .await;

        let result = match update_result {
            Ok(result) => result,
            Err(_) => return Err("database_error".to_string()),
        };

        if result.rows_affected() == 0 {
            return Err("profile_not_found".to_string());
        }
    }

    let audit_result = sqlx::query("insert into audit_logs (user_id, action) values ($1, $2)")
        .bind(user_id)
        .bind("Profile updated")
        .execute(pool)
        .await;

    match audit_result {
        Ok(_) => Ok(()),
        Err(_) => Err("database_error".to_string()),
    }
}

pub async fn change_password(
    pool: &DbPool,
    user_id: i32,
    form: ChangePasswordForm,
) -> Result<(), String> {
    let current_password = form.current_password;
    let new_password = form.new_password;
    let confirm_password = form.confirm_password;

    if new_password.len() < 6 {
        return Err("password_too_short".to_string());
    }

    if new_password != confirm_password {
        return Err("password_mismatch".to_string());
    }

    let user_result = sqlx::query("select password_hash from users where id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await;

    let user = match user_result {
        Ok(Some(row)) => row,
        Ok(None) => return Err("profile_not_found".to_string()),
        Err(_) => return Err("database_error".to_string()),
    };

    let stored_password_hash: String = user.get("password_hash");

    let password_check = auth_service::verify_password(&stored_password_hash, &current_password);

    let password_correct = match password_check {
        Ok(result) => result,
        Err(_) => return Err("database_error".to_string()),
    };

    if !password_correct {
        return Err("current_password_invalid".to_string());
    }

    let new_password_hash = match auth_service::hash_password(&new_password) {
        Ok(hash) => hash,
        Err(_) => return Err("database_error".to_string()),
    };

    let update_result = sqlx::query(
        "update users set password_hash = $1, updated_at = current_timestamp
         where id = $2",
    )
    .bind(&new_password_hash)
    .bind(user_id)
    .execute(pool)
    .await;

    let result = match update_result {
        Ok(result) => result,
        Err(_) => return Err("database_error".to_string()),
    };

    if result.rows_affected() == 0 {
        return Err("profile_not_found".to_string());
    }

    let audit_result = sqlx::query("insert into audit_logs (user_id, action) values ($1, $2)")
        .bind(user_id)
        .bind("Password changed")
        .execute(pool)
        .await;

    match audit_result {
        Ok(_) => Ok(()),
        Err(_) => Err("database_error".to_string()),
    }
}