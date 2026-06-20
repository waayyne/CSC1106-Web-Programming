use rust_decimal::Decimal;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::account::AtmForm;

pub async fn process_atm_transaction(pool: &DbPool, form: AtmForm) -> Result<(), String> {
    let amount = form.amount;

    if amount <= Decimal::ZERO {
        return Err("Amount must be more than 0.".to_string());
    }

    if form.transaction_type != "deposit" && form.transaction_type != "withdraw" {
        return Err("Invalid transaction type.".to_string());
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err("The ATM transaction could not be started.".to_string()),
    };

    let account_result = if form.find_by == "account_number" {
        sqlx::query(
            "SELECT id, balance
             FROM bank_accounts
             WHERE account_number = $1
             FOR UPDATE",
        )
        .bind(&form.account_identifier)
        .fetch_optional(&mut *tx)
        .await
    } else {
        sqlx::query(
            "SELECT ba.id, ba.balance
             FROM bank_accounts ba
             JOIN users u ON ba.user_id = u.id
             WHERE u.phone_number = $1
             FOR UPDATE",
        )
        .bind(&form.account_identifier)
        .fetch_optional(&mut *tx)
        .await
    };

    let account = match account_result {
        Ok(Some(account)) => account,
        Ok(None) => {
            let _ = tx.rollback().await;
            return Err("Account not found.".to_string());
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("We could not find the account.".to_string());
        }
    };

    let account_id: i32 = account.get("id");
    let balance: Decimal = account.get("balance");

    if form.transaction_type == "withdraw" && balance < amount {
        let _ = tx.rollback().await;
        return Err("Insufficient balance.".to_string());
    }

    if form.transaction_type == "deposit" {
        let deposit_result =
            sqlx::query("UPDATE bank_accounts SET balance = balance + $1 WHERE id = $2")
                .bind(amount)
                .bind(account_id)
                .execute(&mut *tx)
                .await;

        if deposit_result.is_err() {
            let _ = tx.rollback().await;
            return Err("The deposit could not be processed.".to_string());
        }

        let save_result = sqlx::query(
            "INSERT INTO transactions
             (from_account_id, to_account_id, transaction_type, amount, description)
             VALUES (NULL, $1, 'deposit', $2, 'ATM deposit')",
        )
        .bind(account_id)
        .bind(amount)
        .execute(&mut *tx)
        .await;

        if save_result.is_err() {
            let _ = tx.rollback().await;
            return Err("An error occurred while saving the transaction.".to_string());
        }
    } else {
        let withdraw_result =
            sqlx::query("UPDATE bank_accounts SET balance = balance - $1 WHERE id = $2")
                .bind(amount)
                .bind(account_id)
                .execute(&mut *tx)
                .await;

        if withdraw_result.is_err() {
            let _ = tx.rollback().await;
            return Err("The withdrawal could not be processed.".to_string());
        }

        let save_result = sqlx::query(
            "INSERT INTO transactions
             (from_account_id, to_account_id, transaction_type, amount, description)
             VALUES ($1, NULL, 'withdraw', $2, 'ATM withdrawal')",
        )
        .bind(account_id)
        .bind(amount)
        .execute(&mut *tx)
        .await;

        if save_result.is_err() {
            let _ = tx.rollback().await;
            return Err("The withdrawal transaction could not be saved.".to_string());
        }
    }

    if tx.commit().await.is_err() {
        return Err("Unable to complete the ATM transaction.".to_string());
    }

    Ok(())
}
