use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use chrono::Local;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tera::Tera;

use crate::db::DbPool;
use crate::services::transaction_service;

const MAX_TRANSACTION_SEARCH_LENGTH: usize = 100;
const MAX_TRANSACTION_PAGE_SIZE: u32 = 100;
const VALID_TRANSACTION_TYPES: &[&str] = &[
    "deposit",
    "withdraw",
    "withdrawal",
    "transfer",
    "fixed_deposit",
    "fixed_deposit_claim",
    "risk_investment",
    "risk_investment_return",
    "loan_disbursement",
];

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TxQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub tx_type: Option<String>,
    pub q: Option<String>,
}

fn validate_transaction_filters(query: &TxQuery) -> Result<(), HttpResponse> {
    for (field_name, value) in [("start_date", &query.start_date), ("end_date", &query.end_date)] {
        if let Some(date_text) = value {
            if chrono::NaiveDate::parse_from_str(date_text, "%Y-%m-%d").is_err() {
                return Err(HttpResponse::BadRequest()
                    .body(format!("{} must use YYYY-MM-DD format.", field_name)));
            }
        }
    }

    if let Some(tx_type) = query.tx_type.as_ref() {
        let normalized = tx_type.trim();
        if !normalized.is_empty() && !VALID_TRANSACTION_TYPES.contains(&normalized) {
            return Err(HttpResponse::BadRequest().body("Invalid transaction type filter."));
        }
    }

    if let Some(search) = query.q.as_ref() {
        if search.chars().count() > MAX_TRANSACTION_SEARCH_LENGTH {
            return Err(HttpResponse::BadRequest().body("Transaction search query is too long."));
        }
    }

    Ok(())
}

pub async fn transactions_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<TxQuery>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

        if let Err(response) = validate_transaction_filters(&query) {
            return response;
        }

    let page = query.page.unwrap_or(1).max(1) as i64;
        let per_page = query
            .per_page
            .unwrap_or(10)
            .clamp(1, MAX_TRANSACTION_PAGE_SIZE) as i64;

    let (transactions, total_count) = match transaction_service::fetch_transactions(
        &pool,
        user_id,
        page,
        per_page,
        query.start_date.clone(),
        query.end_date.clone(),
        query.tx_type.clone(),
        query.q.clone(),
    )
    .await
    {
        Ok(res) => res,
        Err(err) => {
            println!("TRANSACTION LOAD ERROR: {:?}", err);
            return HttpResponse::InternalServerError().body("Transactions failure");
        }
    };

    let cash_flow_summary = match transaction_service::get_cash_flow_summary(&pool, user_id).await {
        Ok(summary) => summary,
        Err(err) => {
            println!("CASH FLOW SUMMARY LOAD ERROR: {:?}", err);
            return HttpResponse::InternalServerError().body("Failed to load cash flow summary");
        }
    };

    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + per_page - 1) / per_page
    };

    let user_row =
        match sqlx::query("SELECT first_name, last_name, username FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => {
                return HttpResponse::Found()
                    .append_header(("Location", "/login"))
                    .finish();
            }
        };

    let account_row =
        match sqlx::query("SELECT account_number, balance FROM bank_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => {
                return HttpResponse::Found()
                    .append_header(("Location", "/login"))
                    .finish();
            }
        };

    let first_name: String = user_row.get("first_name");
    let last_name: String = user_row.get("last_name");
    let account_number: String = account_row.get("account_number");

    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('U'),
        last_name.chars().next().unwrap_or('S')
    );

    let mut context = tera::Context::new();

    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("account_number", &account_number);

    context.insert("transactions", &transactions);
    context.insert("page", &page);
    context.insert("per_page", &per_page);
    context.insert("total_count", &total_count);
    context.insert("total_pages", &total_pages);
    context.insert("total_in", &format!("{:.2}", cash_flow_summary.total_in));
    context.insert("total_out", &format!("{:.2}", cash_flow_summary.total_out));
    context.insert("net_flow", &format!("{:.2}", cash_flow_summary.net_flow));
    context.insert(
        "deposit_total",
        &format!("{:.2}", cash_flow_summary.deposit_total),
    );
    context.insert(
        "withdraw_total",
        &format!("{:.2}", cash_flow_summary.withdraw_total),
    );
    context.insert(
        "transfer_out_total",
        &format!("{:.2}", cash_flow_summary.transfer_out_total),
    );
    context.insert(
        "investment_out_total",
        &format!("{:.2}", cash_flow_summary.investment_out_total),
    );
    context.insert(
        "investment_return_total",
        &format!("{:.2}", cash_flow_summary.investment_return_total),
    );
    context.insert("query", &query.into_inner());

    let rendered = match tmpl.render("transaction_history.html", &context) {
        Ok(s) => s,
        Err(e) => {
            println!("TEMPLATE RENDER ERROR: {:?}", e);
            return HttpResponse::InternalServerError().body("Template render error");
        }
    };

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn transaction_statement_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let user_row = match sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let account_row =
        match sqlx::query("SELECT account_number, balance FROM bank_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => {
                return HttpResponse::Found()
                    .append_header(("Location", "/login"))
                    .finish();
            }
        };

    let first_name: String = user_row.get("first_name");
    let last_name: String = user_row.get("last_name");
    let account_number: String = account_row.get("account_number");
    let current_balance: Decimal = account_row.get("balance");

    let statement_transactions =
        match transaction_service::fetch_statement_transactions(&pool, user_id).await {
            Ok(items) => items,
            Err(err) => {
                println!("STATEMENT LOAD ERROR: {:?}", err);
                return HttpResponse::InternalServerError().body("Statement failed to load");
            }
        };

    let generated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut context = tera::Context::new();

    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("account_number", &account_number);
    context.insert("current_balance", &format!("{:.2}", current_balance));
    context.insert("generated_at", &generated_at);
    context.insert("statement_transactions", &statement_transactions);

    let rendered = match tmpl.render("transaction_statement.html", &context) {
        Ok(s) => s,
        Err(e) => {
            println!("STATEMENT TEMPLATE ERROR: {:?}", e);
            return HttpResponse::InternalServerError().body("Statement template render error");
        }
    };

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/transactions", web::get().to(transactions_page));
    cfg.route(
        "/transactions/statement",
        web::get().to(transaction_statement_page),
    );
}

