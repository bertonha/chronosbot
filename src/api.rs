use actix_web::web::Json;
use actix_web::{Responder, get, post};
use chrono_tz::America::Sao_Paulo;
use chrono_tz::CET;

use crate::command::{convert_from_input_or_default_timezones, process_input};
use crate::telegram::{InlineQueryResult, RequestType, TelegramRequest, TelegramResponse};

#[get("/")]
async fn welcome() -> impl Responder {
    "<h1>Welcome!</h1>"
}

#[post("/")]
async fn receive_message(Json(payload): Json<TelegramRequest>) -> impl Responder {
    let response = match RequestType::from_request(payload) {
        RequestType::Message(message) => {
            if message.is_from_bot() {
                return Json(None);
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
            match convert_from_input_or_default_timezones(inline.query.trim(), &[CET, Sao_Paulo]) {
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
        assert_eq!(data.chat_id, Some(123));
        assert_eq!(data.method, "sendMessage".to_string());
        assert_eq!(data.text, Some("Welcome!\n\nCommands accepted:\n/start\n/now <timezone>\n/convert <time> <source_timezone> <target_timezone>".into()));
    }

    #[tokio::test]
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
        assert_eq!(data.method, "answerInlineQuery".to_string());
        assert_eq!(data.results.unwrap().len(), 2);
    }
}
