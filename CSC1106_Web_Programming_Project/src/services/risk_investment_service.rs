use crate::db::DbPool;
use crate::models::risk_investment::RiskInvestment;

use rand::Rng;
use rust_decimal::Decimal;
use sqlx::Row;

pub async fn create_risk_investment(
    pool: &DbPool,
    user_id: i32,
    amount: Decimal,
    risk_level: String,
) -> Result<(), String> {
    if amount <= Decimal::ZERO {
        return Err("Amount must be more than 0.".to_string());
    }

    let risk_level = risk_level.to_lowercase();

    if risk_level != "low" && risk_level != "medium" && risk_level != "high" {
        return Err("Invalid risk level selected.".to_string());
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Err("Unable to start the investment.".to_string()),
    };

    let account_lookup =
        sqlx::query("select id, balance from bank_accounts where user_id = $1 for update")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await;

    let account_row = match account_lookup {
        Ok(row) => row,
        Err(_) => {
            let _ = tx.rollback().await;
            return Err("Bank account not found.".to_string());
        }
    };

    let account_id: i32 = account_row.get("id");
    let balance: Decimal = account_row.get("balance");

    if balance < amount {
        let _ = tx.rollback().await;
        return Err("Insufficient balance for investment.".to_string());
    }

    let random_number = rand::thread_rng().gen_range(1..=100);

    let (result, return_amount) = match risk_level.as_str() {
        "low" => {
            if random_number <= 75 {
                ("success", amount * Decimal::new(105, 2))
            } else {
                ("failed", Decimal::ZERO)
            }
        }

        "medium" => {
            if random_number <= 60 {
                ("success", amount * Decimal::new(110, 2))
            } else {
                ("failed", Decimal::ZERO)
            }
        }

        "high" => {
            if random_number <= 45 {
                ("success", amount * Decimal::new(120, 2))
            } else {
                ("failed", Decimal::ZERO)
            }
        }

        _ => return Err("Invalid risk level.".to_string()),
    };

    let profit_loss = return_amount - amount;
    let new_balance = balance - amount + return_amount;

    let balance_update = sqlx::query("update bank_accounts set balance = $1 where id = $2")
        .bind(new_balance)
        .bind(account_id)
        .execute(&mut *tx)
        .await;

    if balance_update.is_err() {
        let _ = tx.rollback().await;
        return Err("The account balance could not be updated.".to_string());
    }

    let investment_insert = sqlx::query(
        "insert into risk_investments
         (user_id, account_id, amount, risk_level, result, return_amount, profit_loss)
         values ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(amount)
    .bind(&risk_level)
    .bind(result)
    .bind(return_amount)
    .bind(profit_loss)
    .execute(&mut *tx)
    .await;

    if investment_insert.is_err() {
        let _ = tx.rollback().await;
        return Err("We could not save the risk investment.".to_string());
    }

    let investment_transaction = sqlx::query(
        "insert into transactions
         (from_account_id, to_account_id, transaction_type, amount, description)
         values ($1, null, 'risk_investment', $2, $3)",
    )
    .bind(account_id)
    .bind(amount)
    .bind(format!(
        "Risk investment placed. Risk level: {}. Result: {}.",
        risk_level, result
    ))
    .execute(&mut *tx)
    .await;

    if investment_transaction.is_err() {
        let _ = tx.rollback().await;
        return Err("Unable to record the investment transaction.".to_string());
    }

    if result == "success" {
        let return_transaction = sqlx::query(
            "insert into transactions
             (from_account_id, to_account_id, transaction_type, amount, description)
             values (null, $1, 'risk_investment_return', $2, $3)",
        )
        .bind(account_id)
        .bind(return_amount)
        .bind(format!(
            "Risk investment success. Returned ${:.2}. Profit ${:.2}.",
            return_amount, profit_loss
        ))
        .execute(&mut *tx)
        .await;

        if return_transaction.is_err() {
            let _ = tx.rollback().await;
            return Err("An error occurred while recording the investment return.".to_string());
        }
    }

    if tx.commit().await.is_err() {
        return Err("The investment could not be completed.".to_string());
    }

    Ok(())
}

pub async fn get_risk_investments(
    pool: &DbPool,
    user_id: i32,
) -> Result<Vec<RiskInvestment>, String> {
    let investments = sqlx::query_as::<_, RiskInvestment>(
        "select
            id,
            user_id,
            account_id,
            amount,
            risk_level,
            result,
            return_amount,
            profit_loss,
            created_at
         from risk_investments
         where user_id = $1
         order by created_at desc",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    match investments {
        Ok(investments) => Ok(investments),
        Err(_) => Err("We could not load your risk investments.".to_string()),
    }
}
