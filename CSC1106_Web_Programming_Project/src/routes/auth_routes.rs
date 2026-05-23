use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

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
        .route("/register", web::get().to(register_page))
        .route("/dashboard", web::get().to(dashboard_page));
}