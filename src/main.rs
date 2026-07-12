use actix_web::{App, HttpServer};

use chronosbot::api::{receive_message, welcome};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(welcome).service(receive_message))
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
