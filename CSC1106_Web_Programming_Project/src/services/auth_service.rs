use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use password_hash::{PasswordHash, SaltString};
use rand_core::OsRng;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::user::RegisterForm;

pub async fn register_user(pool: &DbPool, form: RegisterForm) -> Result<(), String> {
    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());

    let hashed_password = match hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => return Err("Failed to hash password.".to_string()),
    };

    let user_result = sqlx::query(
        "insert into users (username, name, first_name, last_name, email, password_hash, phone_number, role)
         values ($1, $2, $3, $4, $5, $6, $7, 'customer') returning id",
    )
    .bind(&form.username)
    .bind(&full_name)
    .bind(&form.first_name)
    .bind(&form.last_name)
    .bind(&form.email)
    .bind(&hashed_password)
    .bind(&form.phone_number)
    .fetch_one(pool)
    .await;

    let user_row = match user_result {
        Ok(row) => row,
        Err(_) => return Err("Failed to register user.".to_string()),
    };

    let user_id: i32 = user_row.get("id");
    let account_number = format!("RB{}", Utc::now().timestamp_millis());

    let account_result = sqlx::query(
        "insert into bank_accounts (user_id, account_number, balance)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(account_number)
    .bind(rust_decimal::Decimal::new(100000, 2))
    .execute(pool)
    .await;

    match account_result {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to create bank account.".to_string()),
    }
}

pub async fn login_user(
    pool: &DbPool,
    identifier: String,
    password: String,
) -> Result<Option<(i32, String)>, String> {
    let identifier = identifier.trim().to_string();

    let user_result = sqlx::query(
        "select id, password_hash, role from users where email = $1 or username = $1",
    )
    .bind(identifier)
    .fetch_optional(pool)
    .await;

    let row = match user_result {
        Ok(row) => row,
        Err(_) => return Err("Failed to check login.".to_string()),
    };

    if let Some(user) = row {
        let user_id: i32 = user.get("id");
        let stored_password: String = user.get("password_hash");
        let role: String = user.get("role");

        let password_correct = match verify_password(&stored_password, &password) {
            Ok(result) => result,
            Err(_) => false,
        };

        if password_correct {
            Ok(Some((user_id, role)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    let result = argon2.verify_password(password.as_bytes(), &parsed_hash);

    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}