use rust_decimal::Decimal;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::transaction::TransferForm;

pub async fn process_transfer(
    pool: &DbPool,
    sender_user_id: i32,
    form: TransferForm,
) -> Result<(), String> {
    let amount = form.amount;

    if amount <= Decimal::ZERO {
        return Err("Amount must be more than 0.".to_string());
    }

    if form.transfer_by != "account_number" && form.transfer_by != "phone_number" {
        return Err("Invalid transfer method.".to_string());
    }

    let sender_result = sqlx::query(
        "SELECT id, account_number
         FROM bank_accounts
         WHERE user_id = $1",
    )
    .bind(sender_user_id)
    .fetch_one(pool)
    .await;

    let sender_account = match sender_result {
        Ok(account) => account,
        Err(_) => return Err("We could not find your bank account.".to_string()),
    };

    let sender_account_id: i32 = sender_account.get("id");
    let sender_account_number: String = sender_account.get("account_number");

    let recipient_result = if form.transfer_by == "account_number" {
        sqlx::query(
            "SELECT id, account_number
             FROM bank_accounts
             WHERE account_number = $1",
        )
        .bind(&form.recipient_identifier)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query(
            "SELECT ba.id, ba.account_number
             FROM bank_accounts ba
             JOIN users u ON ba.user_id = u.id
             WHERE u.phone_number = $1",
        )
        .bind(&form.recipient_identifier)
        .fetch_optional(pool)
        .await
    };

    let recipient_account = match recipient_result {
        Ok(Some(account)) => account,
        Ok(None) => return Err("Recipient account not found.".to_string()),
        Err(_) => return Err("Unable to look up the recipient account.".to_string()),
    };

    let recipient_account_id: i32 = recipient_account.get("id");
    let recipient_account_number: String = recipient_account.get("account_number");

    if sender_account_id == recipient_account_id {
        return Err("You cannot transfer money to your own account.".to_string());
    }

    if sender_account_number == recipient_account_number {
        return Err("You cannot transfer money to your own account.".to_string());
    }

    let description = match form.description {
        Some(text) if !text.trim().is_empty() => text,
        _ => "Bank transfer".to_string(),
    };

    let transaction_result = pool.begin().await;

    let mut tx = match transaction_result {
        Ok(tx) => tx,
        Err(_) => return Err("The transfer could not be started.".to_string()),
    };

    let locked_sender_result = sqlx::query(
        "SELECT id, balance
         FROM bank_accounts
         WHERE id = $1
         FOR UPDATE",
    )
    .bind(sender_account_id)
    .fetch_one(&mut *tx)
    .await;

    let locked_sender = match locked_sender_result {
        Ok(account) => account,
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("Your account could not be prepared for the transfer.".to_string());
        }
    };

    let sender_balance: Decimal = locked_sender.get("balance");

    if sender_balance < amount {
        let _ = tx.rollback().await;
        return Err("Insufficient balance.".to_string());
    }

    let limit_result = sqlx::query(
        "SELECT daily_transfer_limit
     FROM users
     WHERE id = $1",
    )
    .bind(sender_user_id)
    .fetch_one(&mut *tx)
    .await;

    let limit_row = match limit_result {
        Ok(row) => row,
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("Unable to check your daily transfer limit.".to_string());
        }
    };

    let daily_limit: Decimal = limit_row.get("daily_transfer_limit");

    let daily_total_result = sqlx::query(
        "SELECT COALESCE(SUM(amount), 0)::TEXT AS total
        FROM transactions
        WHERE from_account_id = $1
        AND transaction_type = 'transfer'
        AND created_at::date = (now() at time zone 'Asia/Singapore')::date",
    )
    .bind(sender_account_id)
    .fetch_one(&mut *tx)
    .await;

    let daily_total_row = match daily_total_result {
        Ok(row) => row,
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("We could not calculate today's transfer total.".to_string());
        }
    };

    let daily_total_text: String = daily_total_row.get("total");

    let daily_total = match daily_total_text.parse::<Decimal>() {
        Ok(value) => value,
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("Today's transfer total could not be read.".to_string());
        }
    };

    if daily_total + amount > daily_limit {
        let _ = tx.rollback().await;
        return Err(format!(
            "Daily transfer limit exceeded. Your daily limit is ${}. You have already transferred ${} today.",
            daily_limit, daily_total
        ));
    }

    let deduct_result = sqlx::query(
        "UPDATE bank_accounts
         SET balance = balance - $1
         WHERE id = $2",
    )
    .bind(amount)
    .bind(sender_account_id)
    .execute(&mut *tx)
    .await;

    match deduct_result {
        Ok(_) => {}
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("The sender balance could not be updated.".to_string());
        }
    }

    let add_result = sqlx::query(
        "UPDATE bank_accounts
         SET balance = balance + $1
         WHERE id = $2",
    )
    .bind(amount)
    .bind(recipient_account_id)
    .execute(&mut *tx)
    .await;

    match add_result {
        Ok(_) => {}
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("The recipient balance could not be updated.".to_string());
        }
    }

    let save_result = sqlx::query(
        "INSERT INTO transactions (from_account_id, to_account_id, transaction_type, amount, description)
         VALUES ($1, $2, 'transfer', $3, $4)",
    )
    .bind(sender_account_id)
    .bind(recipient_account_id)
    .bind(amount)
    .bind(description)
    .execute(&mut *tx)
    .await;

    match save_result {
        Ok(_) => {}
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("We could not save the transfer transaction.".to_string());
        }
    }

    let commit_result = tx.commit().await;

    match commit_result {
        Ok(_) => Ok(()),
        Err(_) => Err("Unable to complete the transfer.".to_string()),
    }
}
