pub mod events;
pub mod systems;

use serde::{Deserialize, Serialize};
use crate::chat::{ChatHandle, ChatMessageText};

#[derive(Serialize, Deserialize, Debug)]
pub struct OutMessage {
    pub handle: ChatHandle,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub message: ChatMessageText,
    pub handle: ChatHandle,
    // chatId
}
