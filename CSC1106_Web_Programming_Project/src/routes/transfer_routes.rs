use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::transaction::TransferForm;
use crate::services::transfer_service;

#[derive(Serialize)]
struct TransferHistoryItem {
    transaction_type: String,
    amount: String,
    created_at: String,
}

async fn load_user_context(
    pool: &DbPool,
    user_id: i32,
) -> Result<Context, String> {
let user = sqlx::query(
    "SELECT 
        u.first_name,
        u.last_name,
        ba.id AS account_id,
        ba.account_number,
        ba.balance::TEXT AS balance
     FROM users u
     JOIN bank_accounts ba ON u.id = ba.user_id
     WHERE u.id = $1"
)
.bind(user_id)
.fetch_one(pool)
.await
.map_err(|e| {
    println!("LOAD USER CONTEXT ERROR: {:?}", e);
    "Failed to load user details.".to_string()
})?;

    let first_name: String = user.get("first_name");
    let last_name: String = user.get("last_name");
    let account_id: i32 = user.get("account_id");
    let account_number: String = user.get("account_number");
    let balance: String = user.get("balance");

    let history_rows = sqlx::query(
        "SELECT 
            CASE
                WHEN from_account_id = $1 THEN 'Transferred'
                WHEN to_account_id = $1 THEN 'Received'
                ELSE 'Transfer'
            END AS transaction_type,
            amount::TEXT AS amount,
            created_at::TEXT AS created_at
        FROM transactions
        WHERE transaction_type = 'transfer'
        AND (
            from_account_id = $1
            OR to_account_id = $1
        )
        ORDER BY created_at DESC
        LIMIT 5"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        println!("TRANSFER HISTORY ERROR: {:?}", e);
        Vec::new()
    });

    let transfer_history: Vec<TransferHistoryItem> = history_rows
        .into_iter()
        .map(|row| TransferHistoryItem {
            transaction_type: row.get("transaction_type"),
            amount: row.get("amount"),
            created_at: row.get("created_at"),
        })
        .collect();

    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or(' '),
        last_name.chars().next().unwrap_or(' ')
    );

    let mut context = Context::new();

    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("account_number", &account_number);
    context.insert("balance", &balance);
    context.insert("transfer_history", &transfer_history);

    Ok(context)
}

pub async fn transfer_page(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let user_id = session.get::<i32>("user_id").unwrap_or(None);

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let context = match load_user_context(&pool, user_id).await {
        Ok(context) => context,
        Err(error) => {
            return HttpResponse::InternalServerError().body(error);
        }
    };

    let rendered = tmpl.render("transfer_money.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn process_transfer(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    form: web::Form<TransferForm>,
) -> impl Responder {
    let user_id = session.get::<i32>("user_id").unwrap_or(None);

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let result = transfer_service::process_transfer(
        &pool,
        user_id,
        form.into_inner(),
    )
    .await;

    let mut context = match load_user_context(&pool, user_id).await {
        Ok(context) => context,
        Err(error) => {
            return HttpResponse::InternalServerError().body(error);
        }
    };

    match result {
        Ok(_) => {
            context.insert("message", "Transfer completed successfully.");
        }
        Err(error) => {
            context.insert("error", &error);
        }
    }

    let rendered = tmpl.render("transfer_money.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/transfer", web::get().to(transfer_page))
        .route("/transfer", web::post().to(process_transfer));
}