use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::account::AtmForm;
use crate::services::account_service;

pub async fn atm_page(
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let user_id = session.get::<i32>("user_id").unwrap_or(None);

    if user_id.is_none() {
        return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish();
    }

    let mut context = Context::new();

    context.insert("first_name", "Ignatius");
    context.insert("last_name", "Pang");
    context.insert("initials", "IP");
    context.insert("account_number", "RB1779524029824");

    let rendered = tmpl.render("atm.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn process_atm(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    form: web::Form<AtmForm>,
) -> impl Responder {
    let user_id = session.get::<i32>("user_id").unwrap_or(None);

    if user_id.is_none() {
        return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish();
    }

    let result = account_service::process_atm_transaction(
        &pool,
        form.into_inner(),
    )
    .await;

    let mut context = Context::new();

    context.insert("first_name", "Ignatius");
    context.insert("last_name", "Pang");
    context.insert("initials", "IP");
    context.insert("account_number", "RB1779524029824");

    match result {
        Ok(_) => {
            context.insert("message", "ATM transaction completed successfully.");
        }
        Err(error) => {
            context.insert("error", &error);
        }
    }

    let rendered = tmpl.render("atm.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/atm", web::get().to(atm_page))
        .route("/atm", web::post().to(process_atm));
}