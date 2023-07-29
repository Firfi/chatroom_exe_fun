use bevy::prelude::Event;
use crate::chat::message::ChatMessage;

#[derive(Event, Clone)]
pub struct ChatMessageReceivedEvent(pub ChatMessage);
