use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use password_hash::{PasswordHash, SaltString};
use rand::RngCore;
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::env;

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

    let user_result =
        sqlx::query("select id, password_hash, role from users where email = $1 or username = $1")
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

pub async fn request_password_reset(pool: &DbPool, email: String) -> Result<bool, String> {
    let email = email.trim().to_lowercase();

    let user_result = sqlx::query("select id, email from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to check email address.".to_string())?;

    let Some(user) = user_result else {
        return Ok(false);
    };

    let user_id: i32 = user.get("id");
    let user_email: String = user.get("email");
    let token = generate_reset_token();
    let token_hash = hash_reset_token(&token);
    let expires_at = Utc::now().naive_utc() + Duration::minutes(30);

    sqlx::query(
        "update password_reset_tokens
         set used_at = current_timestamp
         where user_id = $1 and used_at is null",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|_| "Failed to replace old reset tokens.".to_string())?;

    sqlx::query(
        "insert into password_reset_tokens (user_id, token_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| "Failed to create reset token.".to_string())?;

    if let Err(error) = send_password_reset_email(&user_email, &token) {
        let _ = sqlx::query(
            "update password_reset_tokens
             set used_at = current_timestamp
             where token_hash = $1 and used_at is null",
        )
        .bind(&token_hash)
        .execute(pool)
        .await;

        return Err(error);
    }

    Ok(true)
}

pub async fn reset_password(pool: &DbPool, token: String, password: String) -> Result<(), String> {
    let token = token.trim().to_string();
    let token_hash = hash_reset_token(&token);

    let token_result = sqlx::query(
        "select id, user_id
         from password_reset_tokens
         where token_hash = $1
           and used_at is null
           and expires_at > current_timestamp",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Failed to check reset token.".to_string())?;

    let Some(token_row) = token_result else {
        return Err("This reset link is invalid or has expired.".to_string());
    };

    let token_id: i32 = token_row.get("id");
    let user_id: i32 = token_row.get("user_id");
    let hashed_password =
        hash_password(&password).map_err(|_| "Failed to hash password.".to_string())?;

    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| "Failed to start password reset.".to_string())?;

    sqlx::query(
        "update users set password_hash = $1, updated_at = current_timestamp where id = $2",
    )
    .bind(&hashed_password)
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| "Failed to update password.".to_string())?;

    sqlx::query("update password_reset_tokens set used_at = current_timestamp where id = $1")
        .bind(token_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| "Failed to mark reset token as used.".to_string())?;

    transaction
        .commit()
        .await
        .map_err(|_| "Failed to finish password reset.".to_string())?;

    Ok(())
}

pub async fn is_reset_token_valid(pool: &DbPool, token: &str) -> Result<bool, String> {
    let token_hash = hash_reset_token(token.trim());

    let result = sqlx::query(
        "select id
         from password_reset_tokens
         where token_hash = $1
           and used_at is null
           and expires_at > current_timestamp",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Failed to check reset token.".to_string())?;

    Ok(result.is_some())
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

fn generate_reset_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_reset_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn send_password_reset_email(to_email: &str, token: &str) -> Result<(), String> {
    let smtp_host =
        env::var("SMTP_HOST").map_err(|_| "SMTP_HOST must be set in .env.".to_string())?;
    let smtp_port = env::var("SMTP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(587);
    let smtp_username =
        env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME must be set in .env.".to_string())?;
    let smtp_password = env::var("SMTP_PASSWORD")
        .map_err(|_| "SMTP_PASSWORD must be set in .env.".to_string())?
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| smtp_username.clone());
    let app_base_url =
        env::var("APP_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let reset_link = format!(
        "{}/reset-password?token={}",
        app_base_url.trim_end_matches('/'),
        token
    );

    let from: Mailbox = smtp_from
        .parse()
        .map_err(|_| "SMTP_FROM must be a valid email address.".to_string())?;
    let to: Mailbox = to_email
        .parse()
        .map_err(|_| "The account email address is invalid.".to_string())?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Reset your RustBank password")
        .body(format!(
            "You requested a RustBank password reset.\n\nOpen this link to set a new password:\n{}\n\nThis link expires in 30 minutes. If you did not request this, you can ignore this email.",
            reset_link
        ))
        .map_err(|_| "Failed to build password reset email.".to_string())?;

    let credentials = Credentials::new(smtp_username, smtp_password);
    let mailer_builder = if smtp_port == 465 {
        SmtpTransport::relay(&smtp_host)
    } else {
        SmtpTransport::starttls_relay(&smtp_host)
    }
    .map_err(|error| format!("Failed to connect to SMTP host: {error}"))?;

    let mailer = mailer_builder
        .port(smtp_port)
        .credentials(credentials)
        .build();

    mailer
        .send(&email)
        .map_err(|error| format!("Failed to send password reset email: {error}"))?;

    Ok(())
}
