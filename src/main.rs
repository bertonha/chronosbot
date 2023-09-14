mod command;
mod telegram;
mod time;

use crate::command::process_command;
use crate::telegram::{TelegramRequest, TelegramResponse};
use axum::{
    routing::{get, post},
    Json, Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"<h1>Welcome!</h1>");
    }
}
