use bevy::prelude::Event;

#[derive(Event)]
pub struct ChatMessageSentStartedEvent(pub String);

#[derive(Event)]
pub struct ChatMessageSentSuccessEvent(pub String);
