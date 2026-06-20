use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::Row;

use crate::db::DbPool;
use crate::models::loan::Loan;

const MAX_LOAN_REASON_LENGTH: usize = 500;

fn max_loan_amount() -> Decimal {
    Decimal::new(999_999_999_999, 2)
}

fn normalize_loan_reason(reason: Option<String>) -> Result<Option<String>, String> {
    match reason {
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if trimmed.chars().count() > MAX_LOAN_REASON_LENGTH {
                Err(format!(
                    "Loan reason must be {} characters or fewer.",
                    MAX_LOAN_REASON_LENGTH
                ))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

#[derive(Serialize)]
pub struct LoanView {
    pub id: i32,
    pub amount: String,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct LoanWithUserView {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub amount: String,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: String,
}

pub async fn apply_for_loan(
    pool: &DbPool,
    user_id: i32,
    amount: Decimal,
    reason: Option<String>,
) -> Result<(), String> {
    if amount <= Decimal::ZERO {
        return Err("Loan amount must be greater than $0.".to_string());
    }

    if amount > max_loan_amount() {
        return Err(format!(
            "Loan amount must be ${:.2} or less.",
            max_loan_amount()
        ));
    }

    let reason = normalize_loan_reason(reason)?;

    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM loans WHERE user_id = $1 AND status = 'pending'")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("We could not check for existing loans: {}", e))?;

    if pending > 0 {
        return Err("You already have a pending loan application.".to_string());
    }

    sqlx::query(
        "INSERT INTO loans (user_id, amount, status, reason) VALUES ($1, $2, 'pending', $3)",
    )
    .bind(user_id)
    .bind(amount)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|e| format!("Your application could not be submitted: {}", e))?;

    Ok(())
}

pub async fn get_user_loans(pool: &DbPool, user_id: i32) -> Result<Vec<LoanView>, String> {
    let loans: Vec<Loan> = sqlx::query_as(
        "SELECT id, user_id, amount, status, reason, created_at
         FROM loans WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Unable to load loans: {}", e))?;

    let loans = loans
        .into_iter()
        .map(|loan| LoanView {
            id: loan.id,
            amount: format!("{:.2}", loan.amount),
            status: loan.status,
            reason: loan.reason,
            created_at: loan.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    Ok(loans)
}

pub async fn get_all_loans(pool: &DbPool) -> Result<Vec<LoanWithUserView>, String> {
    let rows = sqlx::query(
        "SELECT l.id, l.user_id, u.username, u.first_name, u.last_name,
                l.amount, l.status, l.reason, l.created_at
         FROM loans l
         JOIN users u ON l.user_id = u.id
         ORDER BY l.created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("The loan list could not be loaded: {}", e))?;

    let loans = rows
        .into_iter()
        .map(|row| LoanWithUserView {
            id: row.get("id"),
            user_id: row.get("user_id"),
            username: row.get("username"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            amount: format!("{:.2}", row.get::<Decimal, _>("amount")),
            status: row.get("status"),
            reason: row.get("reason"),
            created_at: row
                .get::<NaiveDateTime, _>("created_at")
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        })
        .collect();

    Ok(loans)
}

pub async fn update_loan_status(pool: &DbPool, loan_id: i32, status: &str) -> Result<(), String> {
    if status != "approved" && status != "rejected" {
        return Err("Invalid status value.".to_string());
    }

    if status == "approved" {
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| "Unable to start the loan approval.".to_string())?;

        let loan: Loan = match sqlx::query_as(
            "SELECT id, user_id, amount, status, reason, created_at FROM loans WHERE id = $1 AND status = 'pending' FOR UPDATE",
        )
        .bind(loan_id)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(loan)) => loan,
            Ok(None) => {
                return Err("This loan request is no longer available.".to_string());
            }
            Err(e) => {
                return Err(format!("The selected loan could not be loaded: {}", e));
            }
        };

        let user_id: i32 = loan.user_id;
        let amount: Decimal = loan.amount;

        if amount <= Decimal::ZERO {
            return Err("Loan amount is invalid.".to_string());
        }

        if amount > max_loan_amount() {
            return Err(format!(
                "Loan amount must be ${:.2} or less.",
                max_loan_amount()
            ));
        }

        let account_row = sqlx::query("SELECT id FROM bank_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| "Bank account not found.".to_string())?;

        let account_id: i32 = account_row.get("id");

        sqlx::query("UPDATE bank_accounts SET balance = balance + $1 WHERE id = $2")
            .bind(amount)
            .bind(account_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| "The account could not be credited.".to_string())?;

        sqlx::query(
            "INSERT INTO transactions
             (from_account_id, to_account_id, transaction_type, amount, description)
             VALUES (NULL, $1, 'loan_disbursement', $2, 'Approved loan disbursed')",
        )
            .bind(account_id)
            .bind(amount)
            .execute(&mut *tx)
            .await
            .map_err(|_| "Unable to record the loan disbursement.".to_string())?;

        let status_result = sqlx::query(
            "UPDATE loans SET status = 'approved' WHERE id = $1 AND status = 'pending'",
        )
        .bind(loan_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| "The loan status could not be updated.".to_string())?;

        if status_result.rows_affected() == 0 {
            return Err("This loan request is no longer available.".to_string());
        }

        tx.commit()
            .await
            .map_err(|_| "The loan approval could not be completed.".to_string())?;
    } else {
        let result = sqlx::query("UPDATE loans SET status = 'rejected' WHERE id = $1 AND status = 'pending'")
            .bind(loan_id)
            .execute(pool)
            .await
            .map_err(|e| format!("The loan could not be rejected: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("This loan request is no longer available.".to_string());
        }
    }

    Ok(())
}

