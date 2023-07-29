use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use phf::phf_set;

pub mod message;
pub mod ui;
pub mod systems;
pub mod resources;

// TODO from backend init
pub static CHAT_HANDLES: phf::Set<&'static str> = phf_set! {
    "tete",
    "pepe"
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatHandle(pub String);

impl fmt::Display for ChatHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessageText(pub String);

impl fmt::Display for ChatMessageText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
