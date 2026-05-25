use rust_decimal::Decimal;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::transaction::TransferForm;

pub async fn process_transfer(
    pool: &DbPool,
    sender_user_id: i32,
    form: TransferForm,
) -> Result<(), String> {
    let amount = Decimal::from_f64_retain(form.amount)
        .ok_or("Invalid amount.".to_string())?;

    if amount <= Decimal::ZERO {
        return Err("Amount must be more than 0.".to_string());
    }

    if form.transfer_by != "account_number" && form.transfer_by != "phone_number" {
        return Err("Invalid transfer method.".to_string());
    }

    let sender_account = sqlx::query(
        "SELECT id, account_number, balance 
         FROM bank_accounts 
         WHERE user_id = $1"
    )
    .bind(sender_user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Failed to find your bank account.".to_string())?;

    let sender_account_id: i32 = sender_account.get("id");
    let sender_account_number: String = sender_account.get("account_number");
    let sender_balance: Decimal = sender_account.get("balance");

    let recipient_account = if form.transfer_by == "account_number" {
        sqlx::query(
            "SELECT id, account_number, balance 
             FROM bank_accounts 
             WHERE account_number = $1"
        )
        .bind(&form.recipient_identifier)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to find recipient account.".to_string())?
    } else {
        sqlx::query(
            "SELECT ba.id, ba.account_number, ba.balance
             FROM bank_accounts ba
             JOIN users u ON ba.user_id = u.id
             WHERE u.phone_number = $1"
        )
        .bind(&form.recipient_identifier)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Failed to find recipient account.".to_string())?
    };

    let recipient_account = match recipient_account {
        Some(account) => account,
        None => return Err("Recipient account not found.".to_string()),
    };

    let recipient_account_id: i32 = recipient_account.get("id");
    let recipient_account_number: String = recipient_account.get("account_number");

    if sender_account_id == recipient_account_id {
        return Err("You cannot transfer money to your own account.".to_string());
    }

    if sender_account_number == recipient_account_number {
        return Err("You cannot transfer money to your own account.".to_string());
    }

    if sender_balance < amount {
        return Err("Insufficient balance.".to_string());
    }

    let description = form
        .description
        .unwrap_or_else(|| "Bank transfer".to_string());

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "Failed to start transfer.".to_string())?;

    sqlx::query(
        "UPDATE bank_accounts 
         SET balance = balance - $1 
         WHERE id = $2"
    )
    .bind(amount)
    .bind(sender_account_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| "Failed to deduct sender balance.".to_string())?;

    sqlx::query(
        "UPDATE bank_accounts 
         SET balance = balance + $1 
         WHERE id = $2"
    )
    .bind(amount)
    .bind(recipient_account_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| "Failed to update recipient balance.".to_string())?;

    sqlx::query(
        "INSERT INTO transactions 
         (from_account_id, to_account_id, transaction_type, amount, description)
         VALUES ($1, $2, 'transfer', $3, $4)"
    )
    .bind(sender_account_id)
    .bind(recipient_account_id)
    .bind(amount)
    .bind(description)
    .execute(&mut *tx)
    .await
    .map_err(|_| "Failed to save transfer transaction.".to_string())?;

    tx.commit()
        .await
        .map_err(|_| "Failed to complete transfer.".to_string())?;

    Ok(())
}