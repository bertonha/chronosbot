mod telegram;

use crate::telegram::{TelegramRequest, TelegramResponse};
use axum::{
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::Europe::{Bucharest, Madrid};
use chrono_tz::Tz;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(welcome))
        .route("/", post(receive_message));

    // run it
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn welcome() -> &'static str {
    "<h1>Welcome!</h1>"
}

async fn receive_message(Json(payload): Json<TelegramRequest>) -> Json<Option<TelegramResponse>> {
    if let Some(message) = payload.message() {
        if let Some(message_text) = message.text.as_ref() {
            let text = match message_text.as_str() {
                "/start" => "Welcome!".to_string(),
                "/now utc" => Utc::now().format("%H:%M:%S").to_string(),
                "/now europe" => format_time(Utc::now().with_timezone(&Madrid)),
                "/now brazil" => format_time(Utc::now().with_timezone(&Sao_Paulo)),
                "/now romania" => format_time(Utc::now().with_timezone(&Bucharest)),
                text => parse_time(text),
            };

            return Json(Some(TelegramResponse {
                method: "sendMessage".to_string(),
                chat_id: message.chat.id,
                text,
            }));
        }
    }
    return Json(None);
}

fn parse_time(text: &str) -> String {
    match NaiveTime::parse_from_str(text, "%H:%M:%S") {
        Ok(time) => time.to_string(),
        Err(_) => format!("Invalid time format: {text}").to_string(),
    }
}

fn format_time(time: DateTime<Tz>) -> String {
    time.format("%H:%M:%S").to_string()
}
