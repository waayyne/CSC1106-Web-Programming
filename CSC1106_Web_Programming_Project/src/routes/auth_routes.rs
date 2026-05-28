use crate::db::DbPool;
use crate::middleware::auth_middleware;
use crate::models::user::{LoginForm, RegisterForm};
use crate::services::auth_service;

use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use sqlx::Row;
use tera::{Context, Tera};

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub registered: Option<String>,
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
            Some("Registration successful. Please log in."),
            None,
        );
    }

    render_login_page(&tmpl, None, None, None)
}

pub async fn register_page(tmpl: web::Data<Tera>) -> impl Responder {
    render_register_page(&tmpl, None, None, None, None, None, None)
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

    let first_name = form_data.first_name.clone();
    let last_name = form_data.last_name.clone();
    let username = form_data.username.clone();
    let email = form_data.email.clone();
    let phone_number = form_data.phone_number.clone();

    let result = auth_service::register_user(&pool, form_data).await;

    match result {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/login?registered=1"))
            .finish(),

        Err(_) => render_register_page(
            &tmpl,
            Some("Registration failed. Username, email, or phone number may already exist."),
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
        Ok(Some((user_id, role))) => {
            session.insert("user_id", user_id).unwrap();
            session.insert("role", role.clone()).unwrap(); // clone as session.insert requires ownership of the value

            let redirecrt_url = match role.as_str() { // gets the role as a string slice for matching
                "admin" => "/admin/dashboard",
                "staff" => "/staff/dashboard",
                "customer" => "/dashboard",
                _ => "/dashboard", // default to customer dashboard if role is unrecognized
            };

            HttpResponse::Found()
                .append_header(("Location", redirecrt_url))
                .finish()
        }

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
        .route("/logout", web::get().to(logout))
        .route("/register", web::get().to(register_page))
        .route("/register", web::post().to(register_user))
        .route("/dashboard", web::get().to(dashboard_page));
}
