use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn homepage() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Banking System</title>
            </head>
            <body>
                <h1>Welcome to Banking System</h1>
                <p>Secure online banking built with Rust and Actix Web.</p>

                <a href="/login">Login</a>
                <br>
                <a href="/register">Register</a>
            </body>
            </html>
        "#)
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