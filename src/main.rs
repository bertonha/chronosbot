use axum::{
    routing::{get, post},
    Json, Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::command::process_command;
use crate::telegram::{RequestType, TelegramRequest, TelegramResponse};

mod command;
mod telegram;
mod time;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chronosbot=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = "0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(welcome))
        .route("/", post(receive_message))
        .layer(TraceLayer::new_for_http())
}

async fn welcome() -> &'static str {
    "<h1>Welcome!</h1>"
}

async fn receive_message(Json(payload): Json<TelegramRequest>) -> Json<Option<TelegramResponse>> {
    let response = match RequestType::from_request(payload) {
        RequestType::Message(message) => match message.text {
            Some(text) => Some(TelegramResponse {
                method: "sendMessage".to_string(),
                chat_id: message.chat.id,
                message_id: None,
                text: Some(process_command(&text)),
            }),
            None => None,
        },

        RequestType::EditedMessage(message) => match message.text {
            Some(text) => Some(TelegramResponse {
                method: "editMessageText".to_string(),
                chat_id: message.chat.id,
                message_id: Some(message.message_id + 1),
                text: Some(process_command(&text)),
            }),
            None => None,
        },

        RequestType::InlineQuery(_) => None,

        RequestType::Unknown => None,
    };

    Json(response)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};
    use serde_json::json;
    use tower::util::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn welcome() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Welcome!</h1>");
    }

    #[tokio::test]
    async fn receive_message() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(
                        json!(
                            {
                                "update_id": 123,
                                "message": {
                                    "message_id": 123,
                                    "text": "/start",
                                    "date": 123,
                                    "from": {
                                        "id": 123,
                                        "is_bot": false,
                                        "first_name": "John",
                                    },
                                    "chat": {
                                        "id": 123,
                                        "type": "private",
                                    },
                                }
                            }
                        )
                        .to_string()
                        .into(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
