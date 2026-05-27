use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::middleware::auth_middleware::RequireAuth;
use crate::models::profile::{ChangePasswordForm, UpdateProfileForm};
use crate::services::profile_service::{self, ProfileError};

#[derive(serde::Deserialize)]
pub struct ProfileQuery {
    pub success: Option<String>,
    pub error: Option<String>,
}

fn login_redirect() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
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
    }
}

fn message_for_error(code: &str) -> &'static str {
    match code {
        "profile_not_found" => "Profile not found.",
        "password_too_short" => "New password must be at least 6 characters long.",
        "password_mismatch" => "New password and confirm password do not match.",
        "current_password_invalid" => "Current password is incorrect.",
        "validation_error" => "All profile fields are required.",
        _ => "Something went wrong. Please try again.",
    }
}

fn profile_error_code(error: &ProfileError) -> &'static str {
    error.code()
}

fn extract_user_id(session: &Session) -> Option<i32> {
    session.get::<i32>("user_id").unwrap_or(None)
}

pub async fn profile_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    query: web::Query<ProfileQuery>,
) -> impl Responder {
    let user_id = match extract_user_id(&session) {
        Some(user_id) => user_id,
        None => return login_redirect(),
    };

    let profile = match profile_service::get_profile(&pool, user_id).await {
        Ok(profile) => profile,
        Err(_) => return login_redirect(),
    };

    let initials = format!(
        "{}{}",
        profile.first_name.chars().next().unwrap_or('U'),
        profile.last_name.chars().next().unwrap_or('S')
    );

    let mut context = Context::new();
    context.insert("profile", &profile);
    context.insert("initials", &initials);

    if let Some(code) = query.success.as_deref() {
        context.insert("success_message", message_for_success(code));
    }

    if let Some(code) = query.error.as_deref() {
        context.insert("error_message", message_for_error(code));
    }

    let rendered = tmpl.render("profile_settings.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<UpdateProfileForm>,
) -> impl Responder {
    let user_id = match extract_user_id(&session) {
        Some(user_id) => user_id,
        None => return login_redirect(),
    };

    match profile_service::update_profile(&pool, user_id, form.into_inner()).await {
        Ok(_) => profile_redirect("success=profile_updated"),
        Err(error) => profile_redirect(&format!("error={}", profile_error_code(&error))),
    }
}

pub async fn change_password(
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<ChangePasswordForm>,
) -> impl Responder {
    let user_id = match extract_user_id(&session) {
        Some(user_id) => user_id,
        None => return login_redirect(),
    };

    match profile_service::change_password(&pool, user_id, form.into_inner()).await {
        Ok(_) => profile_redirect("success=password_changed"),
        Err(error) => profile_redirect(&format!("error={}", profile_error_code(&error))),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/profile")
            .wrap(RequireAuth::new())
            .route("", web::get().to(profile_page))
            .route("/update", web::post().to(update_profile))
            .route("/password", web::post().to(change_password)),
    );
}
