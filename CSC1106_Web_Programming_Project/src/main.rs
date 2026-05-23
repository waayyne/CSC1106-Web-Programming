use actix_files::Files;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;
use tera::Tera;

mod db;
mod models;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let db_pool = db::create_pool(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL");

    let tera = Tera::new("templates/**/*").unwrap();

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .service(Files::new("/static", "static"))
            .configure(routes::auth_routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}