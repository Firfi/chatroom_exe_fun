use bevy::prelude::{Deref, DerefMut, Resource, Transform};
use crate::chat::message::ChatMessage;
use crate::chat::ChatHandle;

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Default, Resource)]
pub struct UiState {
    pub chat_input_text: String,
    pub login_input_text: String,
    pub sending_chat_message: bool,
    pub chat_handle: Option<ChatHandle>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct OriginalCameraTransform(pub Transform);
