use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::transaction::TransferForm;
use crate::services::transfer_service;

async fn load_user_context(
    pool: &DbPool,
    user_id: i32,
) -> Result<Context, String> {
    let user = sqlx::query(
        "SELECT 
            u.first_name,
            u.last_name,
            ba.account_number
         FROM users u
         JOIN bank_accounts ba ON u.id = ba.user_id
         WHERE u.id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Failed to load user details.".to_string())?;

    let first_name: String = user.get("first_name");
    let last_name: String = user.get("last_name");
    let account_number: String = user.get("account_number");

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