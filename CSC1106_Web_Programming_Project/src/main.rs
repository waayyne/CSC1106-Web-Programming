use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn homepage() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Banking System")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(homepage))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}