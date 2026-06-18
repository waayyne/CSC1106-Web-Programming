use crate::db::DbPool;
use crate::middleware::auth_middleware;
use crate::models::user::{
    ForgotPasswordForm, LoginForm, RegisterForm, ResendVerificationOtpForm, ResetPasswordForm,
    ResetPasswordQuery, VerifyEmailForm, VerifyEmailQuery,
};
use crate::services::auth_service::{self, LoginResult};

use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use sqlx::Row;
use tera::{Context, Tera};

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub registered: Option<String>,
    pub verified: Option<String>,
}

fn render_login_page(
    tmpl: &Tera,
    error_message: Option<&str>,
    success_message: Option<&str>,
    identifier: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();

    if let Some(message) = error_message {
        context.insert("error_message", message);
    }

    if let Some(message) = success_message {
        context.insert("success_message", message);
    }

    if let Some(value) = identifier {
        context.insert("identifier", value);
    }

    let rendered = tmpl.render("login.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

fn render_register_page(
    tmpl: &Tera,
    error_message: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
    username: Option<&str>,
    email: Option<&str>,
    phone_number: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();

    if let Some(message) = error_message {
        context.insert("error_message", message);
    }

    if let Some(value) = first_name {
        context.insert("first_name", value);
    }

    if let Some(value) = last_name {
        context.insert("last_name", value);
    }

    if let Some(value) = username {
        context.insert("username", value);
    }

    if let Some(value) = email {
        context.insert("email", value);
    }

    if let Some(value) = phone_number {
        context.insert("phone_number", value);
    }

    let rendered = tmpl.render("register.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

fn render_forgot_password_page(
    tmpl: &Tera,
    error_message: Option<&str>,
    success_message: Option<&str>,
    email: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();

    if let Some(message) = error_message {
        context.insert("error_message", message);
    }

    if let Some(message) = success_message {
        context.insert("success_message", message);
    }

    if let Some(value) = email {
        context.insert("email", value);
    }

    let rendered = tmpl.render("forgot_password.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

fn render_reset_password_page(
    tmpl: &Tera,
    token: Option<&str>,
    error_message: Option<&str>,
    success_message: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();

    if let Some(value) = token {
        context.insert("token", value);
    }

    if let Some(message) = error_message {
        context.insert("error_message", message);
    }

    if let Some(message) = success_message {
        context.insert("success_message", message);
    }

    let rendered = tmpl.render("reset_password.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

fn render_verify_email_page(
    tmpl: &Tera,
    error_message: Option<&str>,
    success_message: Option<&str>,
    email: Option<&str>,
) -> HttpResponse {
    let mut context = Context::new();

    if let Some(message) = error_message {
        context.insert("error_message", message);
    }

    if let Some(message) = success_message {
        context.insert("success_message", message);
    }

    if let Some(value) = email {
        context.insert("email", value);
    }

    let rendered = tmpl.render("verify_email.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn homepage(tmpl: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    let rendered = tmpl.render("home.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn login_page(tmpl: web::Data<Tera>, query: web::Query<LoginQuery>) -> impl Responder {
    if query.registered == Some("1".to_string()) {
        return render_login_page(
            &tmpl,
            None,
            Some("Registration successful. Please check your email for the verification OTP."),
            None,
        );
    }

    if query.verified == Some("1".to_string()) {
        return render_login_page(
            &tmpl,
            None,
            Some("Email verified successfully. Please log in."),
            None,
        );
    }

    render_login_page(&tmpl, None, None, None)
}

pub async fn register_page(tmpl: web::Data<Tera>) -> impl Responder {
    render_register_page(&tmpl, None, None, None, None, None, None)
}

pub async fn forgot_password_page(tmpl: web::Data<Tera>) -> impl Responder {
    render_forgot_password_page(&tmpl, None, None, None)
}

pub async fn reset_password_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    query: web::Query<ResetPasswordQuery>,
) -> impl Responder {
    let token = query.token.trim().to_string();

    if token.is_empty() {
        return render_reset_password_page(
            &tmpl,
            None,
            Some("This reset link is missing a token."),
            None,
        );
    }

    match auth_service::is_reset_token_valid(&pool, &token).await {
        Ok(true) => render_reset_password_page(&tmpl, Some(&token), None, None),
        Ok(false) => render_reset_password_page(
            &tmpl,
            None,
            Some("This reset link is invalid or has expired."),
            None,
        ),
        Err(_) => render_reset_password_page(
            &tmpl,
            None,
            Some("Unable to check this reset link. Please try again."),
            None,
        ),
    }
}

pub async fn verify_email_page(
    tmpl: web::Data<Tera>,
    query: web::Query<VerifyEmailQuery>,
) -> impl Responder {
    render_verify_email_page(&tmpl, None, None, query.email.as_deref())
}

pub async fn dashboard_page(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
) -> impl Responder {
    if !auth_middleware::is_logged_in(&session) {
        return auth_middleware::redirect_to_login();
    }

    let user_id = match auth_middleware::get_user_id(&session) {
        Some(user_id) => user_id,
        None => return auth_middleware::redirect_to_login(),
    };

    let user_row = sqlx::query("select first_name, last_name, username from users where id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await;

    let user_row = match user_row {
        Ok(row) => row,
        Err(_) => return auth_middleware::redirect_to_login(),
    };

    let account_row =
        sqlx::query("select account_number, balance from bank_accounts where user_id = $1")
            .bind(user_id)
            .fetch_one(pool.get_ref())
            .await;

    let account_row = match account_row {
        Ok(row) => row,
        Err(_) => return auth_middleware::redirect_to_login(),
    };

    let first_name: String = user_row.get("first_name");
    let last_name: String = user_row.get("last_name");
    let username: String = user_row.get("username");
    let account_number: String = account_row.get("account_number");
    let balance: rust_decimal::Decimal = account_row.get("balance");

    let first_initial = first_name.chars().next().unwrap_or('U');
    let last_initial = last_name.chars().next().unwrap_or('S');
    let initials = format!("{}{}", first_initial, last_initial);

    let mut context = Context::new();

    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("username", &username);
    context.insert("initials", &initials);
    context.insert("account_number", &account_number);
    context.insert("balance", &balance.to_string());

    let rendered = tmpl.render("dashboard.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn register_user(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let form_data = form.into_inner();

    if form_data.password != form_data.confirm_password {
        return render_register_page(
            &tmpl,
            Some("Passwords do not match."),
            Some(&form_data.first_name),
            Some(&form_data.last_name),
            Some(&form_data.username),
            Some(&form_data.email),
            Some(&form_data.phone_number),
        );
    }

    if let Err(message) = auth_service::validate_password_complexity(&form_data.password) {
        return render_register_page(
            &tmpl,
            Some(message),
            Some(&form_data.first_name),
            Some(&form_data.last_name),
            Some(&form_data.username),
            Some(&form_data.email),
            Some(&form_data.phone_number),
        );
    }

    let first_name = form_data.first_name.clone();
    let last_name = form_data.last_name.clone();
    let username = form_data.username.clone();
    let email = form_data.email.clone();
    let phone_number = form_data.phone_number.clone();

    let result = auth_service::register_user(&pool, form_data).await;

    match result {
        Ok(result) => {
            let success_message = if result.otp_email_sent {
                Some("Registration successful. We sent a verification OTP to your email.")
            } else {
                None
            };
            let error_message = result.email_error.as_deref();

            render_verify_email_page(
                &tmpl,
                error_message,
                success_message,
                Some(&result.email),
            )
        }

        Err(message) => render_register_page(
            &tmpl,
            Some(&message),
            Some(&first_name),
            Some(&last_name),
            Some(&username),
            Some(&email),
            Some(&phone_number),
        ),
    }
}

pub async fn login_user(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    session: Session,
    form: web::Form<LoginForm>,
) -> impl Responder {
    let login_form = form.into_inner();

    let identifier = login_form.identifier;
    let password = login_form.password;

    let result = auth_service::login_user(&pool, identifier.clone(), password).await;

    match result {
        Ok(Some(LoginResult::Authenticated { user_id, role })) => {
            session.insert("user_id", user_id).unwrap();
            session.insert("role", role.clone()).unwrap(); 

            let redirecrt_url = match role.as_str() {
                
                "admin" => "/admin/dashboard",
                "staff" => "/staff/dashboard",
                "customer" => "/dashboard",
                _ => "/dashboard", 
            };

            HttpResponse::Found()
                .append_header(("Location", redirecrt_url))
                .finish()
        }

        Ok(Some(LoginResult::EmailNotVerified)) => render_login_page(
            &tmpl,
            Some("Please verify your email before logging in."),
            None,
            Some(&identifier),
        ),

        Ok(None) => render_login_page(
            &tmpl,
            Some("Invalid username/email or password."),
            None,
            Some(&identifier),
        ),

        Err(_) => render_login_page(
            &tmpl,
            Some("Login failed. Please try again."),
            None,
            Some(&identifier),
        ),
    }
}

pub async fn verify_email(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    form: web::Form<VerifyEmailForm>,
) -> impl Responder {
    let form_data = form.into_inner();
    let email = form_data.email.trim().to_string();

    if email.is_empty() || form_data.otp.trim().is_empty() {
        return render_verify_email_page(
            &tmpl,
            Some("Please enter your email and OTP."),
            None,
            Some(&email),
        );
    }

    match auth_service::verify_email_otp(&pool, email.clone(), form_data.otp).await {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/login?verified=1"))
            .finish(),
        Err(message) => render_verify_email_page(&tmpl, Some(&message), None, Some(&email)),
    }
}

pub async fn resend_verification_otp(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    form: web::Form<ResendVerificationOtpForm>,
) -> impl Responder {
    let form_data = form.into_inner();
    let email = form_data.email.trim().to_string();

    if email.is_empty() {
        return render_verify_email_page(
            &tmpl,
            Some("Please enter your email address first."),
            None,
            Some(&email),
        );
    }

    match auth_service::resend_verification_otp(&pool, email.clone()).await {
        Ok(true) => render_verify_email_page(
            &tmpl,
            None,
            Some("A new verification OTP has been sent to your email."),
            Some(&email),
        ),
        Ok(false) => render_verify_email_page(
            &tmpl,
            Some("No unverified account was found for that email."),
            None,
            Some(&email),
        ),
        Err(message) => render_verify_email_page(&tmpl, Some(&message), None, Some(&email)),
    }
}

pub async fn forgot_password(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    form: web::Form<ForgotPasswordForm>,
) -> impl Responder {
    let form_data = form.into_inner();
    let email = form_data.email;

    if email.trim().is_empty() {
        return render_forgot_password_page(
            &tmpl,
            Some("Please enter your account email address."),
            None,
            Some(&email),
        );
    }

    let result = auth_service::request_password_reset(&pool, email.clone()).await;

    match result {
        Ok(true) => render_forgot_password_page(
            &tmpl,
            None,
            Some("A password reset email has been sent. Please check your inbox."),
            None,
        ),
        Ok(false) => render_forgot_password_page(
            &tmpl,
            None,
            Some("If that email exists, a reset link will be sent to it."),
            None,
        ),
        Err(error) => {
            eprintln!("Password reset email error: {error}");
            render_forgot_password_page(
                &tmpl,
                Some("Unable to send the reset email. Please check the mail settings and try again."),
                None,
                Some(&email),
            )
        }
    }
}

pub async fn reset_password(
    tmpl: web::Data<Tera>,
    pool: web::Data<DbPool>,
    form: web::Form<ResetPasswordForm>,
) -> impl Responder {
    let form_data = form.into_inner();
    let token = form_data.token.trim().to_string();

    if form_data.password != form_data.confirm_password {
        return render_reset_password_page(
            &tmpl,
            Some(&token),
            Some("Passwords do not match."),
            None,
        );
    }

    if let Err(message) = auth_service::validate_password_complexity(&form_data.password) {
        return render_reset_password_page(
            &tmpl,
            Some(&token),
            Some(message),
            None,
        );
    }

    match auth_service::reset_password(&pool, token.clone(), form_data.password).await {
        Ok(_) => render_login_page(
            &tmpl,
            None,
            Some("Password reset successful. Please log in with your new password."),
            None,
        ),
        Err(message) => render_reset_password_page(&tmpl, Some(&token), Some(&message), None),
    }
}

pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/login"))
        .finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(homepage))
        .route("/login", web::get().to(login_page))
        .route("/login", web::post().to(login_user))
        .route("/forgot-password", web::get().to(forgot_password_page))
        .route("/forgot-password", web::post().to(forgot_password))
        .route("/reset-password", web::get().to(reset_password_page))
        .route("/reset-password", web::post().to(reset_password))
        .route("/verify-email", web::get().to(verify_email_page))
        .route("/verify-email", web::post().to(verify_email))
        .route(
            "/resend-verification-otp",
            web::post().to(resend_verification_otp),
        )
        .route("/logout", web::get().to(logout))
        .route("/register", web::get().to(register_page))
        .route("/register", web::post().to(register_user))
        .route("/dashboard", web::get().to(dashboard_page));
}
