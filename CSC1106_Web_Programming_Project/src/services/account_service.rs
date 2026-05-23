use rust_decimal::Decimal;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::account::AtmForm;

pub async fn process_atm_transaction(
    pool: &DbPool,
    form: AtmForm,
) -> Result<(), String> {
    let amount = Decimal::from_f64_retain(form.amount)
        .ok_or("Invalid amount.".to_string())?;

    if amount <= Decimal::ZERO {
        return Err("Amount must be more than 0.".to_string());
    }

    let account = if form.find_by == "account_number" {
        sqlx::query(
            "SELECT id, balance FROM bank_accounts WHERE account_number = $1"
        )
        .bind(&form.account_identifier)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to find account.".to_string())?
    } else {
        sqlx::query(
            "SELECT ba.id, ba.balance
             FROM bank_accounts ba
             JOIN users u ON ba.user_id = u.id
             WHERE u.phone_number = $1"
        )
        .bind(&form.account_identifier)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to find account.".to_string())?
    };

    let account = match account {
        Some(account) => account,
        None => return Err("Account not found.".to_string()),
    };

    let account_id: i32 = account.get("id");
    let balance: Decimal = account.get("balance");

    if form.transaction_type == "deposit" {
        sqlx::query(
            "UPDATE bank_accounts SET balance = balance + $1 WHERE id = $2"
        )
        .bind(amount)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|_| "Failed to deposit money.".to_string())?;

        sqlx::query(
            "INSERT INTO transactions 
             (from_account_id, to_account_id, transaction_type, amount, description)
             VALUES (NULL, $1, 'deposit', $2, 'ATM deposit')"
        )
        .bind(account_id)
        .bind(amount)
        .execute(pool)
        .await
        .map_err(|_| "Failed to save transaction.".to_string())?;

        Ok(())
    } else if form.transaction_type == "withdraw" {
        if balance < amount {
            return Err("Insufficient balance.".to_string());
        }

        sqlx::query(
            "UPDATE bank_accounts SET balance = balance - $1 WHERE id = $2"
        )
        .bind(amount)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(|_| "Failed to withdraw money.".to_string())?;

        sqlx::query(
            "INSERT INTO transactions 
             (from_account_id, to_account_id, transaction_type, amount, description)
             VALUES ($1, NULL, 'withdrawal', $2, 'ATM withdrawal')"
        )
        .bind(account_id)
        .bind(amount)
        .execute(pool)
        .await
        .map_err(|_| "Failed to save transaction.".to_string())?;

        Ok(())
    } else {
        Err("Invalid transaction type.".to_string())
    }
}