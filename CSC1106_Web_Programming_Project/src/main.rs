use actix_files::Files;
use actix_web::{web, App, HttpServer};
use tera::Tera;

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("templates/**/*").unwrap();

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .service(Files::new("/static", "static"))
            .configure(routes::auth_routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}