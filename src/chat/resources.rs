use bevy::prelude::{Deref, Resource};
use crossbeam_channel::{Receiver};
use crate::chat::message::ChatMessage;

#[derive(Resource, Deref)]
pub struct StreamReceiver(pub Receiver<ChatMessage>);
