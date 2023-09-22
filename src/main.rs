use actix_web::{App, HttpServer};

use crate::api::{receive_message, welcome};

mod api;
mod command;
mod telegram;
mod time;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(welcome).service(receive_message))
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
