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
    pub message_id: u64,
    pub from: UserPayload,
    pub chat: ChatPayload,
    pub date: u64,
    pub text: String,
    pub entities: Option<Vec<EntityPayload>>,
}

#[derive(Deserialize)]
pub struct UserPayload {
    pub id: u64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub language_code: String,
}

#[derive(Deserialize)]
pub struct ChatPayload {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    #[serde(alias = "type")]
    pub type_: String,
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
    pub chat_id: u64,
    pub text: String,
}
