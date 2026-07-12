use actix_web::web::Json;
use actix_web::{Responder, get, post};
use chrono::{DateTime, Utc};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::{CET, Tz};

use crate::command::{convert_from_input_or_default_timezones, process_input};
use crate::telegram::{InlineQueryResult, RequestType, TelegramRequest, TelegramResponse};

const DEFAULT_INLINE_TIMEZONES: [Tz; 2] = [CET, Sao_Paulo];

#[get("/")]
pub async fn welcome() -> impl Responder {
    "<h1>Welcome!</h1>"
}

#[post("/")]
pub async fn receive_message(Json(payload): Json<TelegramRequest>) -> impl Responder {
    Json(handle_update(payload, Utc::now()))
}

pub fn handle_update(payload: TelegramRequest, now: DateTime<Utc>) -> Option<TelegramResponse> {
    match RequestType::from_request(payload) {
        RequestType::Message(message) => {
            if message.is_from_bot() {
                return None;
            }
            let text = message.text?;
            Some(TelegramResponse::SendMessage {
                chat_id: message.chat.id,
                text: process_input(&text, now),
            })
        }

        RequestType::EditedMessage(message) => {
            let text = message.text?;
            Some(TelegramResponse::EditMessageText {
                chat_id: message.chat.id,
                message_id: message.message_id + 1,
                text: process_input(&text, now),
            })
        }

        RequestType::InlineQuery(inline) => {
            let converter = convert_from_input_or_default_timezones(
                inline.query.trim(),
                &DEFAULT_INLINE_TIMEZONES,
            )
            .ok()?;
            let results = converter
                .convert_time_between_timezones(now)
                .ok()?
                .into_iter()
                .enumerate()
                .map(|(idx, time)| InlineQueryResult::article(idx.to_string(), time))
                .collect();
            Some(TelegramResponse::AnswerInlineQuery {
                inline_query_id: inline.id,
                results,
            })
        }

        RequestType::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{
        App,
        http::{Method, header::ContentType},
        test,
    };
    use serde_json::json;

    #[actix_web::test]
    async fn test_welcome() {
        let app = test::init_service(App::new().service(welcome)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(test::read_body(resp).await, "<h1>Welcome!</h1>");
    }

    #[actix_web::test]
    async fn test_receive_message() {
        let app = test::init_service(App::new().service(receive_message)).await;
        let req = test::TestRequest::default()
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(json!(
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
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let data: TelegramResponse = test::read_body_json(resp).await;
        let TelegramResponse::SendMessage { chat_id, text } = data else {
            panic!("expected sendMessage response, got {data:?}");
        };
        assert_eq!(chat_id, 123);
        assert_eq!(
            text,
            "Welcome!\n\nCommands accepted:\n/start\n/now <timezone>\n/convert <time> <source_timezone> <target_timezone>"
        );
    }

    #[actix_web::test]
    async fn test_receive_inline_message() {
        let app = test::init_service(App::new().service(receive_message)).await;
        let req = test::TestRequest::default()
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(json!(
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
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let data: TelegramResponse = test::read_body_json(resp).await;
        let TelegramResponse::AnswerInlineQuery { results, .. } = data else {
            panic!("expected answerInlineQuery response, got {data:?}");
        };
        assert_eq!(results.len(), 2);
    }
}
