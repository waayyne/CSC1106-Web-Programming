use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::Row;
use tera::Tera;

use crate::db::DbPool;
use crate::services::fixed_deposit_service;

#[derive(Deserialize)]
pub struct FixedDepositForm {
    pub amount: Decimal,
    pub duration_days: i32,
}

#[derive(Deserialize)]
pub struct ClaimForm {
    pub fixed_deposit_id: i32,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub success: Option<String>,
    pub error: Option<String>,
}

pub async fn fixed_deposit_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<MessageQuery>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let user_row = match sqlx::query(
        "SELECT first_name, last_name FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let account_row = match sqlx::query(
        "SELECT account_number, balance FROM bank_accounts WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let first_name: String = user_row.get("first_name");
    let last_name: String = user_row.get("last_name");
    let account_number: String = account_row.get("account_number");
    let balance: Decimal = account_row.get("balance");

    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('U'),
        last_name.chars().next().unwrap_or('S')
    );

    let deposits = match fixed_deposit_service::get_user_fixed_deposits(&pool, user_id).await {
        Ok(list) => list,
        Err(_) => Vec::new(),
    };

    let mut context = tera::Context::new();

    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("account_number", &account_number);
    context.insert("balance", &format!("{:.2}", balance));
    context.insert("deposits", &deposits);
    context.insert("success", &query.success);
    context.insert("error", &query.error);

    let rendered = match tmpl.render("fixed_deposit.html", &context) {
        Ok(html) => html,
        Err(err) => {
            println!("FIXED DEPOSIT TEMPLATE ERROR: {:?}", err);
            return HttpResponse::InternalServerError().body("Template error");
        }
    };

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn create_fixed_deposit(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<FixedDepositForm>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let result = fixed_deposit_service::create_fixed_deposit(
        &pool,
        user_id,
        form.amount,
        form.duration_days,
    )
    .await;

    let redirect_url = match result {
        Ok(_) => "/fixed-deposit?success=Fixed deposit created successfully".to_string(),
        Err(err) => format!("/fixed-deposit?error={}", err),
    };

    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

pub async fn claim_fixed_deposit(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<ClaimForm>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let result = fixed_deposit_service::claim_fixed_deposit(
        &pool,
        user_id,
        form.fixed_deposit_id,
    )
    .await;

    let redirect_url = match result {
        Ok(_) => "/fixed-deposit?success=Fixed deposit claimed successfully".to_string(),
        Err(err) => format!("/fixed-deposit?error={}", err),
    };

    HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/fixed-deposit", web::get().to(fixed_deposit_page));
    cfg.route("/fixed-deposit/create", web::post().to(create_fixed_deposit));
    cfg.route("/fixed-deposit/claim", web::post().to(claim_fixed_deposit));
}