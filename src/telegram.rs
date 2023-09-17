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

#[derive(Deserialize)]
pub struct TelegramRequest {
    pub update_id: u64,
    pub message: Option<Message>,
    pub edited_message: Option<Message>,
    pub inline_query: Option<InlineQuery>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct Entity {
    pub offset: u32,
    pub length: u32,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Deserialize)]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub query: String,
    pub offset: String,
    pub chat_type: String,
}

#[derive(Serialize)]
pub struct TelegramResponse {
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_query_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    results: Option<Vec<InlineQueryResult>>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct InputMessageContent {
    pub message_text: String,
}

impl TelegramResponse {
    pub fn send_message(chat_id: i64, text: String) -> Self {
        Self {
            method: "sendMessage".into(),
            chat_id: Some(chat_id),
            message_id: None,
            text: Some(text),
            inline_query_id: None,
            results: None,
        }
    }

    pub fn edit_message(chat_id: i64, message_id: i64, text: String) -> Self {
        Self {
            method: "editMessageText".into(),
            chat_id: Some(chat_id),
            message_id: Some(message_id),
            text: Some(text),
            inline_query_id: None,
            results: None,
        }
    }

    pub fn answer_inline_query(inline_query_id: String, result: Vec<InlineQueryResult>) -> Self {
        Self {
            method: "answerInlineQuery".into(),
            chat_id: None,
            message_id: None,
            text: None,
            inline_query_id: Some(inline_query_id),
            results: Some(result),
        }
    }
}
