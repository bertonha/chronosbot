use axum::{
    routing::{get, post},
    Json, Router,
};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::CET;
use tower_http::trace::TraceLayer;

use crate::command::{convert_from_input_or_default_timezones, process_input};
use crate::telegram::{InlineQueryResult, RequestType, TelegramRequest, TelegramResponse};

async fn welcome() -> &'static str {
    "<h1>Welcome!</h1>"
}

async fn receive_message(Json(payload): Json<TelegramRequest>) -> Json<Option<TelegramResponse>> {
    tracing::debug!("{:?}", payload);

    let response = match RequestType::from_request(payload) {
        RequestType::Message(message) => {
            if let Some(via_bot) = message.via_bot {
                if via_bot.is_bot {
                    return Json(None);
                }
            }

            match message.text {
                Some(text) => Some(TelegramResponse::send_message(
                    message.chat.id,
                    process_input(&text),
                )),
                None => None,
            }
        }

        RequestType::EditedMessage(message) => match message.text {
            Some(text) => Some(TelegramResponse::edit_message(
                message.chat.id,
                message.message_id + 1,
                process_input(&text),
            )),
            None => None,
        },

        RequestType::InlineQuery(inline) => {
            match convert_from_input_or_default_timezones(inline.query.trim(), vec![CET, Sao_Paulo])
            {
                Ok(converter) => {
                    let results = converter
                        .convert_time_between_timezones()
                        .enumerate()
                        .map(|(idx, time)| InlineQueryResult::article(idx.to_string(), time))
                        .collect::<_>();
                    Some(TelegramResponse::answer_inline_query(inline.id, results))
                }
                Err(_) => None,
            }
        }

        RequestType::Unknown => None,
    };

    Json(response)
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(welcome))
        .route("/", post(receive_message))
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_welcome() {
        let app = app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"<h1>Welcome!</h1>");
    }

    #[tokio::test]
    async fn test_receive_message() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
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
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let data: TelegramResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(data.chat_id, Some(123));
        assert_eq!(data.method, "sendMessage".to_string());
        assert_eq!(data.text, Some("Welcome!\n\nCommands accepted:\n/start\n/now <timezone>\n/convert <time> <source_timezone> <target_timezone>".into()));
    }

    #[tokio::test]
    async fn test_receive_inline_message() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        json!(
                            {
                                "update_id": 123,
                                "inline_query": {
                                    "id": "123",
                                    "from": {
                                        "id": 123,
                                        "is_bot": false,
                                        "first_name": "John",
                                    },
                                    "query": "12",
                                    "offset": "",
                                }
                            }
                        )
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let data: TelegramResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(data.method, "answerInlineQuery".to_string());
        assert_eq!(data.results.unwrap().len(), 2);
    }
}
