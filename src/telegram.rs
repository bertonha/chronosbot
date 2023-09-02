use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TelegramRequest {
    pub update_id: u64,
    pub message: Option<MessagePayload>,
    pub edited_message: Option<MessagePayload>,
}

impl TelegramRequest {
    pub fn message(&self) -> Option<&MessagePayload> {
        if self.edited_message.is_some() {
            self.edited_message.as_ref()
        } else {
            self.message.as_ref()
        }
    }
}

#[derive(Deserialize)]
pub struct MessagePayload {
    pub message_id: i64,
    pub from: UserPayload,
    pub chat: ChatPayload,
    pub date: u64,
    pub text: Option<String>,
    pub new_chat_members: Option<Vec<UserPayload>>,
    pub entities: Option<Vec<EntityPayload>>,
}

#[derive(Deserialize)]
pub struct UserPayload {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: String,
    pub language_code: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatPayload {
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
pub struct EntityPayload {
    pub offset: u32,
    pub length: u32,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Serialize)]
pub struct TelegramResponse {
    pub method: String,
    pub chat_id: i64,
    pub text: String,
}
