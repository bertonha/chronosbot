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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(welcome))
        .route("/", post(receive_message));

    let addr = "0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn welcome() -> &'static str {
    "<h1>Welcome!</h1>"
}

async fn receive_message(Json(payload): Json<TelegramRequest>) -> Json<Option<TelegramResponse>> {
    let response;

    if let Some(message) = payload.message {
        if message.text.is_none() {
            return Json(None);
        }

        response = Some(TelegramResponse {
            method: "sendMessage".to_string(),
            chat_id: message.chat.id,
            message_id: None,
            text: Some(process_command(&message.text.unwrap())),
        });
    } else if let Some(edited_message) = payload.edited_message {
        if edited_message.text.is_none() {
            return Json(None);
        }

        response = Some(TelegramResponse {
            method: "editMessageText".to_string(),
            chat_id: edited_message.chat.id,
            message_id: Some(edited_message.message_id + 1),
            text: Some(process_command(&edited_message.text.unwrap())),
        });
    } else {
        return Json(None);
    }

    Json(response)
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

fn process_command(text: &str) -> String {
    match text {
        "/start" => "Welcome!".to_string(),
        "/now utc" => Utc::now().format("%H:%M:%S").to_string(),
        "/now europe" => format_time(Utc::now().with_timezone(&Madrid)),
        "/now brazil" => format_time(Utc::now().with_timezone(&Sao_Paulo)),
        "/now romania" => format_time(Utc::now().with_timezone(&Bucharest)),
        text => parse_time(text),
    }
}
