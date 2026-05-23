use chrono::Utc;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::user::RegisterForm;

pub async fn register_user(pool: &DbPool, form: RegisterForm) -> Result<(), sqlx::Error> {
    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());

    let user_row = sqlx::query(
        "insert into users (username, name, first_name, last_name, email, password_hash, phone_number, role)
         values ($1, $2, $3, $4, $5, $6, $7, 'customer')
         returning id"
    )
    .bind(&form.username)
    .bind(&full_name)
    .bind(&form.first_name)
    .bind(&form.last_name)
    .bind(&form.email)
    .bind(&form.password)
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

pub async fn login_user(
    pool: &DbPool,
    identifier: String,
    password: String,
) -> Result<Option<i32>, sqlx::Error> {
    let identifier = identifier.trim().to_string();

    let row = sqlx::query(
        "select id, password_hash from users where email = $1 or username = $1"
    )
    .bind(identifier)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = row {
        let user_id: i32 = user.get("id");
        let stored_password: String = user.get("password_hash");

        if stored_password == password {
            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}