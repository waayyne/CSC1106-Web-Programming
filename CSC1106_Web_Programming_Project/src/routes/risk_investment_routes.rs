use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::risk_investment::RiskInvestmentForm;
use crate::services::risk_investment_service;

#[derive(Deserialize)]
pub struct MessageQuery {
    pub success: Option<String>,
    pub error: Option<String>,
}

pub async fn show_risk_investment_page(
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

    let user_row = match sqlx::query("select first_name, last_name from users where id = $1")
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

    let account_row =
        match sqlx::query("select account_number, balance from bank_accounts where user_id = $1")
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

    let investments = risk_investment_service::get_risk_investments(&pool, user_id)
        .await
        .unwrap_or(Vec::new());

    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('U'),
        last_name.chars().next().unwrap_or('S')
    );

    let mut ctx = Context::new();

    ctx.insert("first_name", &first_name);
    ctx.insert("last_name", &last_name);
    ctx.insert("initials", &initials);
    ctx.insert("account_number", &account_number);
    ctx.insert("balance", &format!("{:.2}", balance));
    ctx.insert("investments", &investments);
    ctx.insert("success", &query.success);
    ctx.insert("error", &query.error);

    let rendered = match tmpl.render("risk_investment.html", &ctx) {
        Ok(html) => html,
        Err(err) => {
            println!("Template error: {:?}", err);
            return HttpResponse::InternalServerError().body("Template error");
        }
    };

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn create_risk_investment(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<RiskInvestmentForm>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    let result = risk_investment_service::create_risk_investment(
        &pool,
        user_id,
        form.amount,
        form.risk_level.clone(),
    )
    .await;

    match result {
        Ok(_) => HttpResponse::Found()
            .append_header((
                "Location",
                "/risk-investment?success=Risk investment completed",
            ))
            .finish(),
        Err(error) => {
            let url = format!("/risk-investment?error={}", error);
            HttpResponse::Found()
                .append_header(("Location", url))
                .finish()
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/risk-investment", web::get().to(show_risk_investment_page));
    cfg.route(
        "/risk-investment/create",
        web::post().to(create_risk_investment),
    );
}
