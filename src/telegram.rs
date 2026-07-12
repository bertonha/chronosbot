use serde::{Deserialize, Serialize};

pub enum RequestType {
    Message(Message),
    EditedMessage(Message),
    InlineQuery(InlineQuery),
    Unknown,
}

impl RequestType {
    pub fn from_request(request: TelegramRequest) -> Self {
        if let Some(message) = request.message {
            return Self::Message(message);
        }
        if let Some(edited_message) = request.edited_message {
            return Self::EditedMessage(edited_message);
        }
        if let Some(inline_query) = request.inline_query {
            return Self::InlineQuery(inline_query);
        }
        Self::Unknown
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelegramRequest {
    pub update_id: u64,
    pub message: Option<Message>,
    pub edited_message: Option<Message>,
    pub inline_query: Option<InlineQuery>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub message_id: i64,
    pub from: User,
    pub chat: Chat,
    pub date: u64,
    pub text: Option<String>,
    pub new_chat_members: Option<Vec<User>>,
    pub entities: Option<Vec<Entity>>,
    pub via_bot: Option<User>,
}

impl Message {
    pub fn is_from_bot(&self) -> bool {
        self.from.is_bot || self.via_bot.as_ref().map(|b| b.is_bot).unwrap_or(false)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Chat {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    #[serde(alias = "type")]
    pub type_: String,
    pub title: Option<String>,
    pub all_members_are_administrators: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Entity {
    pub offset: u32,
    pub length: u32,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub query: String,
    pub offset: String,
    pub chat_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", rename_all = "camelCase")]
pub enum TelegramResponse {
    SendMessage {
        chat_id: i64,
        text: String,
    },
    EditMessageText {
        chat_id: i64,
        message_id: i64,
        text: String,
    },
    AnswerInlineQuery {
        inline_query_id: String,
        results: Vec<InlineQueryResult>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InlineQueryResult {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub title: String,
    pub input_message_content: InputMessageContent,
}

impl InlineQueryResult {
    pub fn article(id: String, title: String) -> Self {
        Self {
            type_: "article".into(),
            id,
            title: title.clone(),
            input_message_content: InputMessageContent {
                message_text: title,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessageContent {
    pub message_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_response_serializes_with_method_tag() {
        let response = TelegramResponse::SendMessage {
            chat_id: 5,
            text: "hi".into(),
        };
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            json!({"method": "sendMessage", "chat_id": 5, "text": "hi"})
        );

        let response = TelegramResponse::EditMessageText {
            chat_id: 5,
            message_id: 7,
            text: "hi".into(),
        };
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            json!({"method": "editMessageText", "chat_id": 5, "message_id": 7, "text": "hi"})
        );
    }

    #[test]
    fn test_request_from_request_unknown() {
        let request = TelegramRequest {
            update_id: 0,
            message: None,
            edited_message: None,
            inline_query: None,
        };

        assert!(matches!(
            RequestType::from_request(request),
            RequestType::Unknown
        ));
    }

    #[test]
    fn test_request_from_request_message() {
        let request = TelegramRequest {
            update_id: 0,
            message: Some(Message {
                message_id: 0,
                from: User {
                    id: 0,
                    is_bot: false,
                    first_name: "".into(),
                    last_name: None,
                    username: None,
                    language_code: None,
                },
                chat: Chat {
                    id: 0,
                    first_name: None,
                    last_name: None,
                    username: None,
                    type_: "".into(),
                    title: None,
                    all_members_are_administrators: None,
                },
                date: 0,
                text: None,
                new_chat_members: None,
                entities: None,
                via_bot: None,
            }),
            edited_message: None,
            inline_query: None,
        };

        assert!(matches!(
            RequestType::from_request(request),
            RequestType::Message { .. }
        ));
    }
}
