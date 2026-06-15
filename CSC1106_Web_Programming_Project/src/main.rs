use actix_files::Files;
use actix_session::{ storage::CookieSessionStore, SessionMiddleware };
use actix_web::cookie::Key;
use actix_web::{ web, App, HttpServer };
use dotenvy::dotenv;
use std::env;
use tera::Tera;

// Importing modules for database connection, routes, and services using mod declarations
mod db;
mod middleware;
mod models;
mod routes;
mod services;

#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    let db_pool = db::create_pool(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL");

    let tera = Tera::new("templates/**/*").expect("Failed to load templates");

    let session_key = env::var("SESSION_KEY").expect("SESSION_KEY must be set in .env");

    let secret_key = Key::from(session_key.as_bytes());

    println!("Server running at http://localhost");

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .service(Files::new("/static", "./static"))
            .configure(routes::auth_routes::config)
            .configure(routes::customer_routes::config)
            .configure(routes::profile_routes::config)
            .configure(routes::transfer_routes::config)
            .configure(routes::transaction_routes::config)
            .configure(routes::admin_routes::config)
            .configure(routes::staff_routes::config)
            .configure(routes::fixed_deposit_routes::config)
            .configure(routes::risk_investment_routes::config)
            .configure(routes::loan_routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
