use crate::db::DbPool;
use serde::Serialize;
use sqlx::Row;
use chrono::NaiveDateTime;

#[derive(Serialize)]
pub struct TransactionView {
    pub id: i32,
    pub direction: String,
    pub transaction_type: String,
    pub amount: String,
    pub description: Option<String>,
    pub counterparty: Option<String>,
    pub created_at: String,
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

    // Resolve the logged-in user's bank account id
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

    // Treat an empty tx_type (from the form's "All" selection) as no filter.
    let tx_type_filter: Option<String> = tx_type.and_then(|s| if s.trim().is_empty() { None } else { Some(s) });

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
        SELECT t.id, t.from_account_id, t.to_account_id, t.transaction_type, t.amount::TEXT AS amount,
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

    let rows = sqlx::query(&data_sql)
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
    for row in rows {
        let from_id: Option<i32> = row.get("from_account_id");
        let _to_id: Option<i32> = row.get("to_account_id");
        let amount: String = row.get("amount");
        let tx_type: String = row.get("transaction_type");
        let desc: Option<String> = row.get("description");
        let created_at: NaiveDateTime = row.get("created_at");
        let from_acc: Option<String> = row.get("from_account_number");
        let to_acc: Option<String> = row.get("to_account_number");

        let (direction, counterparty) = if from_id == Some(account_id) {
            ("Out".to_string(), to_acc)
        } else {
            ("In".to_string(), from_acc)
        };

        items.push(TransactionView {
            id: row.get("id"),
            direction,
            transaction_type: tx_type,
            amount,
            description: desc,
            counterparty,
            created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    Ok((items, total_count))
}
