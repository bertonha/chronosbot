use actix_web::{get, post, web, Responder};

use crate::command::{convert, process_command};
use crate::telegram::{InlineQueryResult, RequestType, TelegramRequest, TelegramResponse};

#[get("/")]
async fn welcome() -> &'static str {
    "<h1>Welcome!</h1>"
}

#[post("/")]
async fn receive_message(web::Json(payload): web::Json<TelegramRequest>) -> impl Responder {
    let response = match RequestType::from_request(payload) {
        RequestType::Message(message) => {
            if let Some(via_bot) = message.via_bot {
                if via_bot.is_bot {
                    return web::Json(None);
                }
            }

            match message.text {
                Some(text) => Some(TelegramResponse::send_message(
                    message.chat.id,
                    process_command(&text),
                )),
                None => None,
            }
        }

        RequestType::EditedMessage(message) => match message.text {
            Some(text) => Some(TelegramResponse::edit_message(
                message.chat.id,
                message.message_id + 1,
                process_command(&text),
            )),
            None => None,
        },

        RequestType::InlineQuery(inline) => match convert(inline.query.trim()) {
            Ok(converted) => Some(TelegramResponse::answer_inline_query(
                inline.id,
                vec![InlineQueryResult::article("1".into(), converted)],
            )),
            Err(_) => None,
        },

        RequestType::Unknown => None,
    };

    web::Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{
        http::{header::ContentType, Method},
        test, App,
    };

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
            .set_json(serde_json::json!(
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
}
