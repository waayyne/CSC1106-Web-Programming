use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use password_hash::{PasswordHash, SaltString};
use rand::{Rng, RngCore};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::env;

use crate::db::DbPool;
use crate::models::user::RegisterForm;

pub const PASSWORD_COMPLEXITY_MESSAGE: &str =
    "Password must be at least 8 characters and include uppercase, lowercase, number, and special character.";
const EMAIL_VERIFICATION_OTP_MINUTES: i64 = 10;

pub struct RegistrationResult {
    pub email: String,
    pub otp_email_sent: bool,
    pub email_error: Option<String>,
}

pub enum LoginResult {
    Authenticated { user_id: i32, role: String },
    EmailNotVerified,
}

#[derive(serde::Deserialize)]
struct TurnstileVerificationResponse {
    success: bool,
}

pub fn validate_password_complexity(password: &str) -> Result<(), &'static str> {
    let has_min_length = password.chars().count() >= 8;
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_number = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && !c.is_whitespace());

    if has_min_length && has_uppercase && has_lowercase && has_number && has_special {
        Ok(())
    } else {
        Err(PASSWORD_COMPLEXITY_MESSAGE)
    }
}

pub fn turnstile_site_key() -> Option<String> {
    env::var("TURNSTILE_SITE_KEY").ok()
}

pub async fn verify_captcha(token: &str) -> Result<(), String> {
    let token = token.trim();

    if token.is_empty() {
        return Err("Captcha verification is required.".to_string());
    }

    let secret = env::var("TURNSTILE_SECRET_KEY")
        .map_err(|_| "TURNSTILE_SECRET_KEY must be set in .env.".to_string())?;

    let response = reqwest::Client::new()
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", secret.as_str()), ("response", token)])
        .send()
        .await
        .map_err(|_| "Unable to verify captcha. Please try again.".to_string())?;

    let verification = response
        .json::<TurnstileVerificationResponse>()
        .await
        .map_err(|_| "Unable to read captcha verification response.".to_string())?;

    if verification.success {
        Ok(())
    } else {
        Err("Captcha verification failed. Please try again.".to_string())
    }
}

pub async fn register_user(pool: &DbPool, form: RegisterForm) -> Result<RegistrationResult, String> {
    validate_password_complexity(&form.password).map_err(|message| message.to_string())?;

    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());
    let email = form.email.trim().to_lowercase();

    let hashed_password = match hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => return Err("Failed to hash password.".to_string()),
    };

    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| "Failed to start registration.".to_string())?;

    let user_result = sqlx::query(
        "insert into users (username, name, first_name, last_name, email, password_hash, phone_number, role)
         values ($1, $2, $3, $4, $5, $6, $7, 'customer') returning id",
    )
    .bind(&form.username)
    .bind(&full_name)
    .bind(&form.first_name)
    .bind(&form.last_name)
    .bind(&email)
    .bind(&hashed_password)
    .bind(&form.phone_number)
    .fetch_one(&mut *transaction)
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
    .execute(&mut *transaction)
    .await;

    if account_result.is_err() {
        return Err("Failed to create bank account.".to_string());
    }

    let otp = generate_verification_otp();
    let otp_hash = hash_verification_otp(user_id, &otp);
    let expires_at = Utc::now().naive_utc() + Duration::minutes(EMAIL_VERIFICATION_OTP_MINUTES);

    sqlx::query(
        "insert into email_verification_otps (user_id, otp_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&otp_hash)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(|_| "Failed to create verification OTP.".to_string())?;

    transaction
        .commit()
        .await
        .map_err(|_| "Failed to finish registration.".to_string())?;

    match send_verification_otp_email(&email, &otp) {
        Ok(_) => Ok(RegistrationResult {
            email,
            otp_email_sent: true,
            email_error: None,
        }),
        Err(error) => Ok(RegistrationResult {
            email,
            otp_email_sent: false,
            email_error: Some(error),
        }),
    }
}

pub async fn login_user(
    pool: &DbPool,
    identifier: String,
    password: String,
) -> Result<Option<LoginResult>, String> {
    let identifier = identifier.trim().to_string();

    let user_result = sqlx::query(
        "select id, password_hash, role, email_verified from users where email = $1 or username = $1",
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
        let email_verified: bool = user.get("email_verified");

        let password_correct = match verify_password(&stored_password, &password) {
            Ok(result) => result,
            Err(_) => false,
        };

        if password_correct {
            if email_verified {
                Ok(Some(LoginResult::Authenticated { user_id, role }))
            } else {
                Ok(Some(LoginResult::EmailNotVerified))
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn verify_email_otp(pool: &DbPool, email: String, otp: String) -> Result<(), String> {
    let email = email.trim().to_lowercase();
    let otp = otp.trim();

    if !is_valid_otp_format(otp) {
        return Err("Please enter the 6-digit OTP sent to your email.".to_string());
    }

    let user_result = sqlx::query("select id, email_verified from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to check email address.".to_string())?;

    let Some(user) = user_result else {
        return Err("Invalid email or OTP.".to_string());
    };

    let user_id: i32 = user.get("id");
    let email_verified: bool = user.get("email_verified");

    if email_verified {
        return Ok(());
    }

    let otp_hash = hash_verification_otp(user_id, otp);
    let otp_result = sqlx::query(
        "select id
         from email_verification_otps
         where user_id = $1
           and otp_hash = $2
           and used_at is null
           and expires_at > current_timestamp
         order by created_at desc
         limit 1",
    )
    .bind(user_id)
    .bind(&otp_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| "Failed to check verification OTP.".to_string())?;

    let Some(otp_row) = otp_result else {
        return Err("Invalid or expired OTP.".to_string());
    };

    let otp_id: i32 = otp_row.get("id");
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| "Failed to start email verification.".to_string())?;

    sqlx::query(
        "update users
         set email_verified = true,
             email_verified_at = current_timestamp,
             updated_at = current_timestamp
         where id = $1",
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| "Failed to verify email.".to_string())?;

    sqlx::query("update email_verification_otps set used_at = current_timestamp where id = $1")
        .bind(otp_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| "Failed to mark OTP as used.".to_string())?;

    transaction
        .commit()
        .await
        .map_err(|_| "Failed to finish email verification.".to_string())?;

    Ok(())
}

pub async fn resend_verification_otp(pool: &DbPool, email: String) -> Result<bool, String> {
    let email = email.trim().to_lowercase();

    let user_result = sqlx::query("select id, email_verified from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to check email address.".to_string())?;

    let Some(user) = user_result else {
        return Ok(false);
    };

    let user_id: i32 = user.get("id");
    let email_verified: bool = user.get("email_verified");

    if email_verified {
        return Ok(false);
    }

    create_and_send_verification_otp(pool, user_id, &email).await?;
    Ok(true)
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
    validate_password_complexity(&password).map_err(|message| message.to_string())?;

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

fn generate_verification_otp() -> String {
    let otp = rand::thread_rng().gen_range(0..1_000_000);
    format!("{:06}", otp)
}

fn hash_reset_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn hash_verification_otp(user_id: i32, otp: &str) -> String {
    let hash = Sha256::digest(format!("{}:{}", user_id, otp.trim()).as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn is_valid_otp_format(otp: &str) -> bool {
    otp.len() == 6 && otp.chars().all(|character| character.is_ascii_digit())
}

async fn create_and_send_verification_otp(
    pool: &DbPool,
    user_id: i32,
    email: &str,
) -> Result<(), String> {
    let otp = generate_verification_otp();
    let otp_hash = hash_verification_otp(user_id, &otp);
    let expires_at = Utc::now().naive_utc() + Duration::minutes(EMAIL_VERIFICATION_OTP_MINUTES);

    sqlx::query(
        "update email_verification_otps
         set used_at = current_timestamp
         where user_id = $1 and used_at is null",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|_| "Failed to replace old verification OTPs.".to_string())?;

    sqlx::query(
        "insert into email_verification_otps (user_id, otp_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&otp_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| "Failed to create verification OTP.".to_string())?;

    send_verification_otp_email(email, &otp)
}

fn send_verification_otp_email(to_email: &str, otp: &str) -> Result<(), String> {
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

    let from: Mailbox = smtp_from
        .parse()
        .map_err(|_| "SMTP_FROM must be a valid email address.".to_string())?;
    let to: Mailbox = to_email
        .parse()
        .map_err(|_| "The account email address is invalid.".to_string())?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Verify your WIVAH Bank email")
        .body(format!(
            "Welcome to WIVAH Bank.\n\nYour email verification OTP is: {}\n\nThis OTP expires in {} minutes. If you did not create this account, you can ignore this email.",
            otp, EMAIL_VERIFICATION_OTP_MINUTES
        ))
        .map_err(|_| "Failed to build verification email.".to_string())?;

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
        .map_err(|error| format!("Failed to send verification email: {error}"))?;

    Ok(())
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
        .subject("Reset your WIVAH Bank password")
        .body(format!(
            "You requested a WIVAH Bank password reset.\n\nOpen this link to set a new password:\n{}\n\nThis link expires in 30 minutes. If you did not request this, you can ignore this email.",
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
