use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
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

pub fn validate_password_complexity(password: &str) -> Result<(), &'static str> {
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_number = false;
    let mut has_special = false;

    for character in password.chars() {
        if character.is_ascii_uppercase() {
            has_uppercase = true;
        } else if character.is_ascii_lowercase() {
            has_lowercase = true;
        } else if character.is_ascii_digit() {
            has_number = true;
        } else if !character.is_whitespace() {
            has_special = true;
        }
    }

    let has_min_length = password.chars().count() >= 8;

    if has_min_length && has_uppercase && has_lowercase && has_number && has_special {
        Ok(())
    } else {
        Err(PASSWORD_COMPLEXITY_MESSAGE)
    }
}

pub async fn register_user(
    pool: &DbPool,
    form: RegisterForm,
) -> Result<RegistrationResult, String> {
    match validate_password_complexity(&form.password) {
        Ok(_) => {}
        Err(message) => return Err(message.to_string()),
    }

    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());
    let email = form.email.trim().to_lowercase();

    let hashed_password = match hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => return Err("The password could not be secured.".to_string()),
    };

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err("Registration could not be started.".to_string()),
    };

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
        Err(error) => {
            let error_text = error.to_string();

            if error_text.contains("users_username_key") {
                return Err("Username already exists.".to_string());
            }

            if error_text.contains("users_email_key") {
                return Err("Email already exists.".to_string());
            }

            if error_text.contains("users_phone_number_key") {
                return Err("Phone number already exists.".to_string());
            }

            return Err(format!("An error occurred while registering the user: {}", error_text));
        }
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

    match account_result {
        Ok(_) => {}
        Err(_) => return Err("We could not create the bank account.".to_string()),
    }

    let otp = generate_verification_otp();
    let expires_at = Utc::now().naive_utc() + Duration::minutes(EMAIL_VERIFICATION_OTP_MINUTES);

    let otp_insert = sqlx::query(
        "insert into email_verification_otps (user_id, otp_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&otp)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await;

    match otp_insert {
        Ok(_) => {}
        Err(_) => return Err("Unable to create the verification OTP.".to_string()),
    }

    match transaction.commit().await {
        Ok(_) => {}
        Err(_) => return Err("Registration could not be completed.".to_string()),
    }

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
    .bind(&identifier)
    .fetch_optional(pool)
    .await;

    let row = match user_result {
        Ok(row) => row,
        Err(_) => return Err("We could not verify your login details.".to_string()),
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

    let user_lookup = sqlx::query("select id, email_verified from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await;

    let user = match user_lookup {
        Ok(Some(user)) => user,
        Ok(None) => return Err("Invalid email or OTP.".to_string()),
        Err(_) => return Err("Unable to check the email address.".to_string()),
    };

    let user_id: i32 = user.get("id");
    let email_verified: bool = user.get("email_verified");

    if email_verified {
        return Ok(());
    }

    let otp_lookup = sqlx::query(
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
    .bind(otp)
    .fetch_optional(pool)
    .await;

    let otp_row = match otp_lookup {
        Ok(Some(row)) => row,
        Ok(None) => return Err("Invalid or expired OTP.".to_string()),
        Err(_) => return Err("We could not validate the verification OTP.".to_string()),
    };

    let otp_id: i32 = otp_row.get("id");

    let verify_result = sqlx::query(
        "update users
         set email_verified = true,
             email_verified_at = current_timestamp,
             updated_at = current_timestamp
         where id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await;

    match verify_result {
        Ok(_) => {}
        Err(_) => return Err("The email address could not be verified.".to_string()),
    }

    let mark_otp_result =
        sqlx::query("update email_verification_otps set used_at = current_timestamp where id = $1")
            .bind(otp_id)
            .execute(pool)
            .await;

    match mark_otp_result {
        Ok(_) => {}
        Err(_) => return Err("Unable to finalize the verification code.".to_string()),
    }

    Ok(())
}

pub async fn resend_verification_otp(pool: &DbPool, email: String) -> Result<bool, String> {
    let email = email.trim().to_lowercase();

    let user_lookup = sqlx::query("select id, email_verified from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await;

    let user = match user_lookup {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(false),
        Err(_) => return Err("We could not check the email address.".to_string()),
    };

    let user_id: i32 = user.get("id");
    let email_verified: bool = user.get("email_verified");

    if email_verified {
        return Ok(false);
    }

    match create_and_send_verification_otp(pool, user_id, &email).await {
        Ok(_) => {}
        Err(message) => return Err(message),
    }

    Ok(true)
}

pub async fn request_password_reset(pool: &DbPool, email: String) -> Result<bool, String> {
    let email = email.trim().to_lowercase();

    let user_lookup = sqlx::query("select id, email from users where lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await;

    let user = match user_lookup {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(false),
        Err(_) => return Err("The email address could not be checked.".to_string()),
    };

    let user_id: i32 = user.get("id");
    let user_email: String = user.get("email");
    let token = generate_reset_token();
    let token_hash = hash_reset_token(&token);
    let expires_at = Utc::now().naive_utc() + Duration::minutes(30);

    let old_token_update = sqlx::query(
        "update password_reset_tokens
         set used_at = current_timestamp
         where user_id = $1 and used_at is null",
    )
    .bind(user_id)
    .execute(pool)
    .await;

    match old_token_update {
        Ok(_) => {}
        Err(_) => return Err("Unable to prepare a new password reset link.".to_string()),
    }

    let token_insert = sqlx::query(
        "insert into password_reset_tokens (user_id, token_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await;

    match token_insert {
        Ok(_) => {}
        Err(_) => return Err("A new password reset link could not be created.".to_string()),
    }

    let email_result = send_password_reset_email(&user_email, &token);

    match email_result {
        Ok(_) => {}
        Err(error) => {
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
    }

    Ok(true)
}

pub async fn reset_password(pool: &DbPool, token: String, password: String) -> Result<(), String> {
    match validate_password_complexity(&password) {
        Ok(_) => {}
        Err(message) => return Err(message.to_string()),
    }

    let token = token.trim().to_string();
    let token_hash = hash_reset_token(&token);

    let token_lookup = sqlx::query(
        "select id, user_id
         from password_reset_tokens
         where token_hash = $1
           and used_at is null
           and expires_at > current_timestamp",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await;

    let token_row = match token_lookup {
        Ok(Some(row)) => row,
        Ok(None) => return Err("This reset link is invalid or has expired.".to_string()),
        Err(_) => return Err("We could not validate the reset link.".to_string()),
    };

    let token_id: i32 = token_row.get("id");
    let user_id: i32 = token_row.get("user_id");
    let hashed_password = match hash_password(&password) {
        Ok(hash) => hash,
        Err(_) => return Err("The password could not be secured.".to_string()),
    };

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Err("The password reset could not be started.".to_string()),
    };

    let password_update = sqlx::query(
        "update users set password_hash = $1, updated_at = current_timestamp where id = $2",
    )
    .bind(&hashed_password)
    .bind(user_id)
    .execute(&mut *transaction)
    .await;

    match password_update {
        Ok(_) => {}
        Err(_) => return Err("Your password could not be updated.".to_string()),
    }

    let token_update =
        sqlx::query("update password_reset_tokens set used_at = current_timestamp where id = $1")
            .bind(token_id)
            .execute(&mut *transaction)
            .await;

    match token_update {
        Ok(_) => {}
        Err(_) => return Err("Unable to finalize the password reset link.".to_string()),
    }

    match transaction.commit().await {
        Ok(_) => {}
        Err(_) => return Err("The password reset could not be completed.".to_string()),
    }

    Ok(())
}

pub async fn is_reset_token_valid(pool: &DbPool, token: &str) -> Result<bool, String> {
    let token_hash = hash_reset_token(token.trim());

    let token_lookup = sqlx::query(
        "select id
         from password_reset_tokens
         where token_hash = $1
           and used_at is null
           and expires_at > current_timestamp",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await;

    match token_lookup {
        Ok(result) => Ok(result.is_some()),
        Err(_) => Err("Unable to validate the reset link.".to_string()),
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

fn is_valid_otp_format(otp: &str) -> bool {
    otp.len() == 6 && otp.chars().all(|character| character.is_ascii_digit())
}

async fn create_and_send_verification_otp(
    pool: &DbPool,
    user_id: i32,
    email: &str,
) -> Result<(), String> {
    let otp = generate_verification_otp();
    let expires_at = Utc::now().naive_utc() + Duration::minutes(EMAIL_VERIFICATION_OTP_MINUTES);

    let old_otp_update = sqlx::query(
        "update email_verification_otps
         set used_at = current_timestamp
         where user_id = $1 and used_at is null",
    )
    .bind(user_id)
    .execute(pool)
    .await;

    match old_otp_update {
        Ok(_) => {}
        Err(_) => return Err("We could not prepare a new verification OTP.".to_string()),
    }

    let otp_insert = sqlx::query(
        "insert into email_verification_otps (user_id, otp_hash, expires_at)
         values ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&otp)
    .bind(expires_at)
    .execute(pool)
    .await;

    match otp_insert {
        Ok(_) => {}
        Err(_) => return Err("The verification OTP could not be created.".to_string()),
    }

    match send_verification_otp_email(email, &otp) {
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    }
}

fn send_verification_otp_email(to_email: &str, otp: &str) -> Result<(), String> {
    let body = format!(
        "Welcome to WIVAH Bank.\n\nYour email verification OTP is: {}\n\nThis OTP expires in {} minutes. If you did not create this account, you can ignore this email.",
        otp, EMAIL_VERIFICATION_OTP_MINUTES
    );

    send_email(
        to_email,
        "Verify your WIVAH Bank email",
        body,
        "verification email",
    )
}

fn send_password_reset_email(to_email: &str, token: &str) -> Result<(), String> {
    let app_base_url =
        env::var("APP_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let reset_link = format!(
        "{}/reset-password?token={}",
        app_base_url.trim_end_matches('/'),
        token
    );

    let body = format!(
        "You requested a WIVAH Bank password reset.\n\nOpen this link to set a new password:\n{}\n\nThis link expires in 30 minutes. If you did not request this, you can ignore this email.",
        reset_link
    );

    send_email(
        to_email,
        "Reset your WIVAH Bank password",
        body,
        "password reset email",
    )
}

fn send_email(to_email: &str, subject: &str, body: String, email_type: &str) -> Result<(), String> {
    let smtp_host = match env::var("SMTP_HOST") {
        Ok(value) => value,
        Err(_) => return Err("SMTP_HOST must be set in .env.".to_string()),
    };

    let smtp_port_text = match env::var("SMTP_PORT") {
        Ok(value) => value,
        Err(_) => "587".to_string(),
    };

    let smtp_port = match smtp_port_text.parse::<u16>() {
        Ok(value) => value,
        Err(_) => 587,
    };

    let smtp_username = match env::var("SMTP_USERNAME") {
        Ok(value) => value,
        Err(_) => return Err("SMTP_USERNAME must be set in .env.".to_string()),
    };

    let smtp_password_text = match env::var("SMTP_PASSWORD") {
        Ok(value) => value,
        Err(_) => return Err("SMTP_PASSWORD must be set in .env.".to_string()),
    };

    let mut smtp_password = String::new();
    for character in smtp_password_text.chars() {
        if !character.is_whitespace() {
            smtp_password.push(character);
        }
    }

    let smtp_from = match env::var("SMTP_FROM") {
        Ok(value) => value,
        Err(_) => smtp_username.clone(),
    };

    let from: Mailbox = match smtp_from.parse() {
        Ok(mailbox) => mailbox,
        Err(_) => return Err("SMTP_FROM must be a valid email address.".to_string()),
    };
    let to: Mailbox = match to_email.parse() {
        Ok(mailbox) => mailbox,
        Err(_) => return Err("The account email address is invalid.".to_string()),
    };

    let email_result = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body);

    let email = match email_result {
        Ok(email) => email,
        Err(_) => return Err(format!("The {email_type} could not be prepared.")),
    };

    let mailer_result = if smtp_port == 465 {
        SmtpTransport::relay(&smtp_host)
    } else {
        SmtpTransport::starttls_relay(&smtp_host)
    };

    let mailer_builder = match mailer_result {
        Ok(builder) => builder,
        Err(error) => return Err(format!("Unable to connect to the email service: {}", error)),
    };

    let credentials = Credentials::new(smtp_username, smtp_password);
    let mailer = mailer_builder
        .port(smtp_port)
        .credentials(credentials)
        .build();

    let send_result = mailer.send(&email);

    match send_result {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("The {} could not be sent: {}", email_type, error)),
    }
}
