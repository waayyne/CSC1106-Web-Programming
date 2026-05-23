use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};
use crate::db::DbPool;
use crate::services::auth_service;
use crate::models::user::{RegisterForm, LoginForm};
pub async fn homepage() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"
            <h1>Welcome to Banking System</h1>
            <p>Rust + Actix Web online banking project</p>
            <a href="/login">Login</a><br>
            <a href="/register">Register</a><br>
            <a href="/dashboard">Test Dashboard</a>
        "#)
}

pub async fn login_page(tmpl: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    let rendered = tmpl.render("login.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn register_page(tmpl: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    let rendered = tmpl.render("register.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub async fn dashboard_page(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    context.insert("name", "Test User");
    context.insert("balance", "1000.00");
    context.insert("account_number", "RB100001");

    let rendered = tmpl.render("customer_dashboard.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(homepage))
        .route("/login", web::get().to(login_page))
        .route("/login", web::post().to(login_user))
        .route("/register", web::get().to(register_page))
        .route("/register", web::post().to(register_user))
        .route("/dashboard", web::get().to(dashboard_page));
}

pub async fn register_user(
    pool: web::Data<DbPool>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let result = auth_service::register_user(&pool, form.into_inner()).await;

    match result {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish(),
        Err(_) => HttpResponse::Ok()
            .content_type("text/html")
            .body("Registration failed. Email or phone number may already exist."),
    }
}

pub async fn login_user(
    pool: web::Data<DbPool>,
    form: web::Form<LoginForm>,
) -> impl Responder {
    let login_form = form.into_inner();

    let result = auth_service::login_user(
        &pool,
        login_form.email,
        login_form.password,
    )
    .await;

    match result {
        Ok(true) => HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
        Ok(false) => HttpResponse::Ok()
            .content_type("text/html")
            .body("Invalid email or password."),
        Err(_) => HttpResponse::Ok()
            .content_type("text/html")
            .body("Login failed."),
    }
}