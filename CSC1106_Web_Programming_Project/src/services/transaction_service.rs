use crate::db::DbPool;
use crate::models::transaction::{CashFlowSummary, TransactionRecord};

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::Row;

#[derive(Serialize)]
pub struct TransactionView {
    pub id: i32,
    pub direction: String,
    pub transaction_type: String,
    pub transaction_type_label: String,
    pub amount: String,
    pub description: Option<String>,
    pub counterparty: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct StatementTransactionView {
    pub id: i32,
    pub direction: String,
    pub transaction_type: String,
    pub transaction_type_label: String,
    pub amount: String,
    pub description: Option<String>,
    pub counterparty: Option<String>,
    pub created_at: String,
    pub balance_after: String,
}

pub async fn fetch_transactions(
    pool: &DbPool,
    user_id: i32,
    page: i64,
    per_page: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    tx_type: Option<String>,
    q: Option<String>,
) -> Result<(Vec<TransactionView>, i64), sqlx::Error> {
    let offset = (page - 1) * per_page;

    let account_row = sqlx::query("SELECT id FROM bank_accounts WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    let account_id: i32 = match account_row {
        Some(r) => r.get("id"),
        None => return Ok((Vec::new(), 0)),
    };

    let start_ts: Option<NaiveDateTime> = start_date
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap());

    let end_ts: Option<NaiveDateTime> = end_date
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(23, 59, 59).unwrap());

    let tx_type_filter: Option<String> =
        tx_type.and_then(|s| if s.trim().is_empty() { None } else { Some(s) });

    let count_sql = r#"
        SELECT COUNT(*) FROM transactions t
        LEFT JOIN bank_accounts bf ON t.from_account_id = bf.id
        LEFT JOIN bank_accounts bt ON t.to_account_id = bt.id
        WHERE (t.from_account_id = $1 OR t.to_account_id = $1)
          AND ($2::text IS NULL OR t.transaction_type = $2)
          AND ($3::text IS NULL OR (t.description ILIKE '%' || $3 || '%') OR (bf.account_number ILIKE '%' || $3 || '%') OR (bt.account_number ILIKE '%' || $3 || '%'))
          AND ($4::timestamp IS NULL OR t.created_at >= $4)
          AND ($5::timestamp IS NULL OR t.created_at <= $5)
    "#;

    let total_row = sqlx::query(count_sql)
        .bind(account_id)
        .bind(tx_type_filter.clone())
        .bind(q.clone())
        .bind(start_ts)
        .bind(end_ts)
        .fetch_one(pool)
        .await?;

    let total_count: i64 = total_row.get::<i64, _>(0);

    let data_sql = r#"
        SELECT t.id, t.from_account_id, t.to_account_id, t.transaction_type, t.amount,
               t.description, t.created_at,
               bf.account_number AS from_account_number,
               bt.account_number AS to_account_number
        FROM transactions t
        LEFT JOIN bank_accounts bf ON t.from_account_id = bf.id
        LEFT JOIN bank_accounts bt ON t.to_account_id = bt.id
        WHERE (t.from_account_id = $1 OR t.to_account_id = $1)
          AND ($2::text IS NULL OR t.transaction_type = $2)
          AND ($3::text IS NULL OR (t.description ILIKE '%' || $3 || '%') OR (bf.account_number ILIKE '%' || $3 || '%') OR (bt.account_number ILIKE '%' || $3 || '%'))
          AND ($4::timestamp IS NULL OR t.created_at >= $4)
          AND ($5::timestamp IS NULL OR t.created_at <= $5)
        ORDER BY t.created_at DESC
        LIMIT $6 OFFSET $7
    "#;

    let records: Vec<TransactionRecord> = sqlx::query_as(data_sql)
        .bind(account_id)
        .bind(tx_type_filter)
        .bind(q)
        .bind(start_ts)
        .bind(end_ts)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let mut items = Vec::new();

    for r in records {
        let direction = get_transaction_direction(&r, account_id);
        let counterparty = get_transaction_counterparty(&r, account_id);

        items.push(TransactionView {
            id: r.id,
            direction,
            transaction_type_label: format_transaction_type_label(&r.transaction_type),
            transaction_type: r.transaction_type,
            amount: format!("{:.2}", r.amount),
            description: r.description,
            counterparty,
            created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    Ok((items, total_count))
}

pub async fn fetch_statement_transactions(
    pool: &DbPool,
    user_id: i32,
) -> Result<Vec<StatementTransactionView>, sqlx::Error> {
    let account_row = sqlx::query(
        "SELECT id, balance FROM bank_accounts WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let account_row = match account_row {
        Some(row) => row,
        None => return Ok(Vec::new()),
    };

    let account_id: i32 = account_row.get("id");
    let current_balance: Decimal = account_row.get("balance");

    let data_sql = r#"
        SELECT t.id, t.from_account_id, t.to_account_id, t.transaction_type, t.amount,
               t.description, t.created_at,
               bf.account_number AS from_account_number,
               bt.account_number AS to_account_number
        FROM transactions t
        LEFT JOIN bank_accounts bf ON t.from_account_id = bf.id
        LEFT JOIN bank_accounts bt ON t.to_account_id = bt.id
        WHERE (t.from_account_id = $1 OR t.to_account_id = $1)
        ORDER BY t.created_at ASC
    "#;

    let records: Vec<TransactionRecord> = sqlx::query_as(data_sql)
        .bind(account_id)
        .fetch_all(pool)
        .await?;

    let mut total_effect = Decimal::ZERO;

    for r in &records {
        total_effect += get_transaction_effect(r, account_id);
    }

    let mut running_balance = current_balance - total_effect;
    let mut items = Vec::new();

    for r in records {
        let effect = get_transaction_effect(&r, account_id);
        running_balance += effect;

        let direction = get_transaction_direction(&r, account_id);
        let counterparty = get_transaction_counterparty(&r, account_id);

        items.push(StatementTransactionView {
            id: r.id,
            direction,
            transaction_type_label: format_transaction_type_label(&r.transaction_type),
            transaction_type: r.transaction_type,
            amount: format!("{:.2}", r.amount),
            description: r.description,
            counterparty,
            created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            balance_after: format!("{:.2}", running_balance),
        });
    }

    Ok(items)
}

fn get_transaction_effect(tx: &TransactionRecord, account_id: i32) -> Decimal {
    let tx_type = tx.transaction_type.to_lowercase();

    if tx_type == "deposit" {
        tx.amount
    } else if tx_type == "withdraw" || tx_type == "withdrawal" {
        -tx.amount
    } else if tx.from_account_id == Some(account_id) {
        -tx.amount
    } else if tx.to_account_id == Some(account_id) {
        tx.amount
    } else {
        Decimal::ZERO
    }
}

fn get_transaction_direction(tx: &TransactionRecord, account_id: i32) -> String {
    if tx.to_account_id == Some(account_id) {
        "In".to_string()
    } else if tx.from_account_id == Some(account_id) {
        "Out".to_string()
    } else {
        "-".to_string()
    }
}

fn get_transaction_counterparty(
    tx: &TransactionRecord,
    account_id: i32,
) -> Option<String> {
    if tx.to_account_id == Some(account_id) {
        tx.from_account_number.clone()
    } else if tx.from_account_id == Some(account_id) {
        tx.to_account_number.clone()
    } else {
        None
    }
}

fn format_transaction_type_label(transaction_type: &str) -> String {
    match transaction_type.to_lowercase().as_str() {
        "deposit" => "Deposit".to_string(),
        "withdraw" | "withdrawal" => "Withdraw".to_string(),
        "transfer" => "Transfer".to_string(),
        "fixed_deposit" => "Fixed Deposit".to_string(),
        "fixed_deposit_claim" => "Fixed Deposit Claim".to_string(),
        "risk_investment" => "Risk Investment".to_string(),
        "risk_investment_return" => "Risk Investment Return".to_string(),
        other => other
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" "),
    }
}

pub async fn get_cash_flow_summary(
    pool: &DbPool,
    user_id: i32,
) -> Result<CashFlowSummary, String> {
    let account_row = sqlx::query("SELECT id FROM bank_accounts WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|err| err.to_string())?;

    let account_id = match account_row {
        Some(row) => row.get::<i32, _>("id"),
        None => {
            return Ok(CashFlowSummary {
                total_in: Decimal::ZERO,
                total_out: Decimal::ZERO,
                net_flow: Decimal::ZERO,
                deposit_total: Decimal::ZERO,
                withdraw_total: Decimal::ZERO,
                transfer_out_total: Decimal::ZERO,
                investment_out_total: Decimal::ZERO,
                investment_return_total: Decimal::ZERO,
            });
        }
    };

    let total_in: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE to_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let total_out: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE from_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let deposit_total: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE transaction_type = 'deposit'
          AND to_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let withdraw_total: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE transaction_type = 'withdraw'
          AND from_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let transfer_out_total: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE transaction_type = 'transfer'
          AND from_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let investment_out_total: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE transaction_type = 'risk_investment'
          AND from_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    let investment_return_total: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM transactions
        WHERE transaction_type = 'risk_investment_return'
          AND to_account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|err| err.to_string())?;

    Ok(CashFlowSummary {
        total_in,
        total_out,
        net_flow: total_in - total_out,
        deposit_total,
        withdraw_total,
        transfer_out_total,
        investment_out_total,
        investment_return_total,
    })
}