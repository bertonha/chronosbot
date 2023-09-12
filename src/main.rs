mod command;
mod telegram;
mod time;

use axum::{
    routing::{get, post},
    Json, Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::telegram::{TelegramRequest, TelegramResponse};

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

fn process_command(text: &str) -> String {
    match text {
        "/start" => command::start(),

        _ => match text.split_once(' ') {
            Some((command, rest)) => match command {
                "/now" => command::now(rest),
                "/convert" => command::convert_time(rest).unwrap_or_else(|e| e.to_string()),
                _ => "Invalid command".to_string(),
            },
            None => "Invalid command".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_command_start() {
        let result = process_command("/start");
        assert_eq!(result, "Welcome!".to_string());
    }
    #[test]
    fn test_process_command_now() {
        let result = process_command("/now utc");
        assert_eq!(result, command::now("utc"));
    }
    #[test]
    fn test_process_command_convert() {
        let result = process_command("/convert 12:00 UTC BRT");
        assert_eq!(result, "09:00:00".to_string());
    }
}
