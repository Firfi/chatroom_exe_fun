use bevy::prelude::Event;
use crate::chat::ChatHandle;

#[derive(Event)]
pub struct LoggedIn(pub ChatHandle);
