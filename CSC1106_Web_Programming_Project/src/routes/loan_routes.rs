use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::services::{audit_service, loan_service};

#[derive(Deserialize)]
pub struct LoanForm {
    pub amount: Decimal,
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct LoanActionForm {
    pub loan_id: i32,
    pub action: String,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub success: Option<String>,
    pub error: Option<String>,
}

fn require_staff(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;
    if role == "staff" { Some(user_id) } else { None }
}



pub async fn loan_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<MessageQuery>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/login")).finish(),
    };

    let user_row = match sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::Found().append_header(("Location", "/login")).finish(),
    };

    let account_row =
        match sqlx::query("SELECT account_number, balance FROM bank_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(r) => r,
            Err(_) => return HttpResponse::Found().append_header(("Location", "/login")).finish(),
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

    let loans = loan_service::get_user_loans(&pool, user_id)
    .await
    .unwrap_or_default();

    let pending_loan_count = loans
        .iter()
        .filter(|loan| loan.status == "pending")
        .count();

    let mut ctx = Context::new();
    ctx.insert("first_name", &first_name);
    ctx.insert("last_name", &last_name);
    ctx.insert("initials", &initials);
    ctx.insert("account_number", &account_number);
    ctx.insert("balance", &format!("{:.2}", balance));
    ctx.insert("loans", &loans);
    ctx.insert("pending_loan_count", &pending_loan_count);
    ctx.insert("success", &query.success);
    ctx.insert("error", &query.error);

    let rendered = match tmpl.render("loan_application.html", &ctx) {
        Ok(html) => html,
        Err(e) => {
            println!("Loan template error: {:?}", e);
            return HttpResponse::InternalServerError().body("Template error");
        }
    };
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn apply_loan(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<LoanForm>,
) -> impl Responder {
    let user_id = match session.get::<i32>("user_id").unwrap_or(None) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/login")).finish(),
    };

    let result =
        loan_service::apply_for_loan(&pool, user_id, form.amount, form.reason.clone()).await;

    match result {
        Ok(_) => {
            let _ =
                audit_service::log_action(&pool, Some(user_id), "Submitted loan application")
                    .await;
            HttpResponse::Found()
                .append_header(("Location", "/loans?success=Loan+application+submitted"))
                .finish()
        }
        Err(e) => {
            let encoded = e.replace(' ', "+");
            HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("/loans?error={}", encoded),
                ))
                .finish()
        }
    }
}



pub async fn staff_loans_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<MessageQuery>,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/dashboard"))
                .finish()
        }
    };

    let staff_row = match sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(staff_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::Found().append_header(("Location", "/login")).finish(),
    };

    let first_name: String = staff_row.get("first_name");
    let last_name: String = staff_row.get("last_name");
    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('S'),
        last_name.chars().next().unwrap_or('T')
    );
    let role = session.get::<String>("role").unwrap_or(None).unwrap_or_default();

    let loans = loan_service::get_all_loans(&pool).await.unwrap_or_default();

    let pending_count = loans.iter().filter(|l| l.status == "pending").count();

    let mut ctx = Context::new();
    ctx.insert("first_name", &first_name);
    ctx.insert("last_name", &last_name);
    ctx.insert("initials", &initials);
    ctx.insert("role", &role);
    ctx.insert("loans", &loans);
    ctx.insert("pending_count", &pending_count);
    ctx.insert("success", &query.success);
    ctx.insert("error", &query.error);

    let rendered = match tmpl.render("manage_loans.html", &ctx) {
        Ok(html) => html,
        Err(e) => {
            println!("manage_loans template error: {:?}", e);
            return HttpResponse::InternalServerError().body("Template error");
        }
    };
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn handle_loan_action(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<LoanActionForm>,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => {
            return HttpResponse::Found()
                .append_header(("Location", "/dashboard"))
                .finish()
        }
    };

    let new_status = match form.action.as_str() {
        "approve" => "approved",
        "reject" => "rejected",
        _ => {
            return HttpResponse::Found()
                .append_header(("Location", "/staff/loans?error=Invalid+action"))
                .finish()
        }
    };

    let result = loan_service::update_loan_status(&pool, form.loan_id, new_status).await;

    match result {
        Ok(_) => {
            let msg = format!("Staff {} {} loan {}", staff_id, new_status, form.loan_id);
            let _ = audit_service::log_action(&pool, Some(staff_id), &msg).await;
            let redirect = format!("/staff/loans?success=Loan+{}", new_status);
            HttpResponse::Found()
                .append_header(("Location", redirect))
                .finish()
        }
        Err(e) => {
            let encoded = e.replace(' ', "+");
            HttpResponse::Found()
                .append_header(("Location", format!("/staff/loans?error={}", encoded)))
                .finish()
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/loans", web::get().to(loan_page))
        .route("/loans/apply", web::post().to(apply_loan))
        .route("/staff/loans", web::get().to(staff_loans_page))
    .route("/staff/loans/action", web::post().to(handle_loan_action));
}