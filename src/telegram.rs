use serde::{Deserialize, Serialize};

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
    pub method: String,
    pub chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
