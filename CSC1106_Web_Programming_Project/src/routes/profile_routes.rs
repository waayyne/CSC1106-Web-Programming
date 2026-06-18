use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};
use sqlx::Row;

use crate::db::DbPool;
use crate::middleware::auth_middleware;
use crate::models::profile::{ChangePasswordForm, UpdateProfileForm, UpdateTransferLimitForm};
use crate::services::auth_service;
use crate::services::profile_service;

#[derive(serde::Deserialize)]
pub struct ProfileQuery {
    pub success: Option<String>,
    pub error: Option<String>,
}

fn profile_redirect(query: &str) -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", format!("/profile?{}", query)))
        .finish()
}

fn message_for_success(code: &str) -> &'static str {
    match code {
        "profile_updated" => "Profile updated successfully.",
        "password_changed" => "Password changed successfully.",
        _ => "Success.",
        "limit_updated" => "Daily transfer limit updated successfully.",
    }
}

fn message_for_error(code: &str) -> &'static str {
    match code {
        "profile_not_found" => "Profile not found.",
        "password_complexity" => auth_service::PASSWORD_COMPLEXITY_MESSAGE,
        "password_mismatch" => "New password and confirm password do not match.",
        "current_password_invalid" => "Current password is incorrect.",
        "validation_error" => "All profile fields are required.",
        "database_error" => "Something went wrong. Please try again.",
        _ => "Something went wrong. Please try again.",
        "limit_invalid" => "Daily transfer limit must be more than 0.",
    }
}

pub async fn profile_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<ProfileQuery>,
) -> impl Responder {
    let user_id = match auth_middleware::get_user_id(&session) {
        Some(user_id) => user_id,
        None => return auth_middleware::redirect_to_login(),
    };

    let profile = match profile_service::get_profile(&pool, user_id).await {
        Ok(profile) => profile,
        Err(_) => return auth_middleware::redirect_to_login(),
    };

    let first_initial = profile.first_name.chars().next().unwrap_or('U');
    let last_initial = profile.last_name.chars().next().unwrap_or('S');
    let initials = format!("{}{}", first_initial, last_initial);

    let mut context = Context::new();

    context.insert("profile", &profile);
    context.insert("initials", &initials);

    // Try to load the user's account number for header display
    let account_row = sqlx::query("select account_number from bank_accounts where user_id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await;

    if let Ok(row) = account_row {
        let account_number: String = row.get("account_number");
        context.insert("account_number", &account_number);
    }

    if let Some(code) = &query.success {
        context.insert("success_message", message_for_success(code));
    }

    if let Some(code) = &query.error {
        context.insert("error_message", message_for_error(code));
    }

    let rendered = tmpl.render("profile_settings.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn update_transfer_limit(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<UpdateTransferLimitForm>,
) -> impl Responder {
    let user_id = match auth_middleware::get_user_id(&session) {
        Some(user_id) => user_id,
        None => return auth_middleware::redirect_to_login(),
    };

    let result = profile_service::update_transfer_limit(&pool, user_id, form.into_inner()).await;

    match result {
        Ok(_) => profile_redirect("success=limit_updated"),
        Err(error) => profile_redirect(&format!("error={}", error)),
    }
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<UpdateProfileForm>,
) -> impl Responder {
    let user_id = match auth_middleware::get_user_id(&session) {
        Some(user_id) => user_id,
        None => return auth_middleware::redirect_to_login(),
    };

    let result = profile_service::update_profile(&pool, user_id, form.into_inner()).await;

    match result {
        Ok(_) => profile_redirect("success=profile_updated"),
        Err(error) => profile_redirect(&format!("error={}", error)),
    }
}

pub async fn change_password(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<ChangePasswordForm>,
) -> impl Responder {
    let user_id = match auth_middleware::get_user_id(&session) {
        Some(user_id) => user_id,
        None => return auth_middleware::redirect_to_login(),
    };

    let result = profile_service::change_password(&pool, user_id, form.into_inner()).await;

    match result {
        Ok(_) => profile_redirect("success=password_changed"),
        Err(error) => profile_redirect(&format!("error={}", error)),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/profile")
            .route("", web::get().to(profile_page))
            .route("/update", web::post().to(update_profile))
            .route("/transfer-limit", web::post().to(update_transfer_limit))
            .route("/password", web::post().to(change_password)),
    );
}
