use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::user::RegisterForm;

pub async fn register_user(pool: &DbPool, form: RegisterForm) -> Result<(), sqlx::Error> {
    let password_hash = hash(form.password, DEFAULT_COST)
        .expect("Failed to hash password");

    let user_row = sqlx::query(
        "insert into users (name, email, password_hash, phone_number, role)
         values ($1, $2, $3, $4, 'customer')
         returning id"
    )
    .bind(&form.name)
    .bind(&form.email)
    .bind(&password_hash)
    .bind(&form.phone_number)
    .fetch_one(pool)
    .await?;

    let user_id: i32 = user_row.get("id");

    let account_number = format!("RB{}", Utc::now().timestamp_millis());

    sqlx::query(
        "insert into bank_accounts (user_id, account_number, balance)
         values ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(account_number)
    .bind(rust_decimal::Decimal::new(100000, 2))
    .execute(pool)
    .await?;

    Ok(())
}


pub async fn login_user(pool: &DbPool, email: String, password: String) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "select password_hash from users where email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = row {
        let password_hash: String = user.get("password_hash");

        let is_valid = verify(password, &password_hash)
            .unwrap_or(false);

        Ok(is_valid)
    } else {
        Ok(false)
    }
}