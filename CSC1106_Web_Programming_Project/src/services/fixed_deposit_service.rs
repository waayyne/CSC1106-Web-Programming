use crate::db::DbPool;
use crate::models::fixed_deposit::FixedDeposit;

use chrono::{Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sqlx::Row;

fn singapore_now() -> chrono::NaiveDateTime {
    let singapore_time = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&singapore_time).naive_local()
}

pub async fn create_fixed_deposit(
    pool: &DbPool,
    user_id: i32,
    amount: Decimal,
    duration_days: i32,
) -> Result<(), String> {
    if amount <= Decimal::ZERO {
        return Err("Amount must be more than $0.".to_string());
    }

    let account_lookup = sqlx::query("SELECT id, balance FROM bank_accounts WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await;

    let account_row = match account_lookup {
        Ok(row) => row,
        Err(_) => return Err("Bank account not found.".to_string()),
    };

    let account_id: i32 = account_row.get("id");
    let balance: Decimal = account_row.get("balance");

    if balance < amount {
        return Err("Insufficient balance for fixed deposit.".to_string());
    }

    let maturity_seconds = match duration_days {
        90 => 9,
        180 => 18,
        360 => 36,
        _ => return Err("Invalid fixed deposit duration.".to_string()),
    };

    let interest_rate = match duration_days {
        90 => Decimal::new(150, 2),
        180 => Decimal::new(250, 2),
        360 => Decimal::new(350, 2),
        _ => return Err("Invalid fixed deposit duration.".to_string()),
    };

    let interest_amount = amount * interest_rate / Decimal::new(100, 0);
    let total_return = amount + interest_amount;

    let now = singapore_now();
    let maturity_at = now + Duration::seconds(maturity_seconds.into());

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err("Failed to start database transaction.".to_string()),
    };

    let deduct_result =
        sqlx::query("UPDATE bank_accounts SET balance = balance - $1 WHERE id = $2")
            .bind(amount)
            .bind(account_id)
            .execute(&mut *tx)
            .await;

    if deduct_result.is_err() {
        return Err("Failed to deduct account balance.".to_string());
    }

    sqlx::query(
        "INSERT INTO fixed_deposits
         (user_id, account_id, principal_amount, interest_rate, interest_amount, total_return,
          duration_days, maturity_seconds, status, created_at, maturity_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active', $9, $10)",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(amount)
    .bind(interest_rate)
    .bind(interest_amount)
    .bind(total_return)
    .bind(duration_days)
    .bind(maturity_seconds)
    .bind(now)
    .bind(maturity_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to create fixed deposit: {}", e))?;

    let transaction_result = sqlx::query(
        "INSERT INTO transactions
         (from_account_id, to_account_id, transaction_type, amount, description)
         VALUES ($1, NULL, 'fixed_deposit', $2, $3)",
    )
    .bind(account_id)
    .bind(amount)
    .bind(format!("Fixed deposit created for {} days", duration_days))
    .execute(&mut *tx)
    .await;

    if transaction_result.is_err() {
        return Err("Failed to record transaction.".to_string());
    }

    if tx.commit().await.is_err() {
        return Err("Failed to save fixed deposit.".to_string());
    }

    Ok(())
}

pub async fn get_user_fixed_deposits(
    pool: &DbPool,
    user_id: i32,
) -> Result<Vec<FixedDeposit>, String> {
    let now = singapore_now();

    let status_update = sqlx::query(
        "UPDATE fixed_deposits
         SET status = 'done'
         WHERE user_id = $1
           AND status = 'active'
           AND maturity_at <= $2",
    )
    .bind(user_id)
    .bind(now)
    .execute(pool)
    .await;

    if status_update.is_err() {
        return Err("Failed to update fixed deposit status.".to_string());
    }

    let deposit_lookup = sqlx::query_as::<_, FixedDeposit>(
        "SELECT * FROM fixed_deposits
         WHERE user_id = $1
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    match deposit_lookup {
        Ok(deposits) => Ok(deposits),
        Err(_) => Err("Failed to load fixed deposits.".to_string()),
    }
}

pub async fn claim_fixed_deposit(
    pool: &DbPool,
    user_id: i32,
    fixed_deposit_id: i32,
) -> Result<(), String> {
    let now = singapore_now();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err("Failed to start database transaction.".to_string()),
    };

    let deposit_lookup = sqlx::query_as::<_, FixedDeposit>(
        "SELECT *
         FROM fixed_deposits
         WHERE id = $1 AND user_id = $2
         FOR UPDATE",
    )
    .bind(fixed_deposit_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await;

    let deposit = match deposit_lookup {
        Ok(deposit) => deposit,
        Err(_) => return Err("Fixed deposit not found.".to_string()),
    };

    if deposit.status == "claimed" {
        let _ = tx.rollback().await;
        return Err("This fixed deposit has already been claimed.".to_string());
    }

    if now < deposit.maturity_at {
        let _ = tx.rollback().await;
        return Err("Fixed deposit has not matured yet.".to_string());
    }

    let return_result =
        sqlx::query("UPDATE bank_accounts SET balance = balance + $1 WHERE id = $2")
            .bind(deposit.total_return)
            .bind(deposit.account_id)
            .execute(&mut *tx)
            .await;

    if return_result.is_err() {
        return Err("Failed to return fixed deposit amount.".to_string());
    }

    let status_update = sqlx::query(
        "UPDATE fixed_deposits
         SET status = 'claimed', claimed_at = $1
         WHERE id = $2 AND user_id = $3 AND status != 'claimed'",
    )
    .bind(now)
    .bind(fixed_deposit_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await;

    let update_result = match status_update {
        Ok(result) => result,
        Err(_) => return Err("Failed to update fixed deposit status.".to_string()),
    };

    if update_result.rows_affected() == 0 {
        let _ = tx.rollback().await;
        return Err("This fixed deposit has already been claimed.".to_string());
    }

    let claim_transaction = sqlx::query(
        "INSERT INTO transactions
         (from_account_id, to_account_id, transaction_type, amount, description)
         VALUES (NULL, $1, 'fixed_deposit_claim', $2, $3)",
    )
    .bind(deposit.account_id)
    .bind(deposit.total_return)
    .bind(format!(
        "Fixed deposit claimed. Interest earned: ${:.2}",
        deposit.interest_amount
    ))
    .execute(&mut *tx)
    .await;

    if claim_transaction.is_err() {
        return Err("Failed to record claim transaction.".to_string());
    }

    if tx.commit().await.is_err() {
        return Err("Failed to claim fixed deposit.".to_string());
    }

    Ok(())
}