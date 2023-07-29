
use async_trait::async_trait;
use bevy::utils::tracing;
use crossbeam_channel::Sender;
use ezsockets::Error;
use crate::chat::ChatHandle;
use crate::chat::message::{ChatMessage, OutMessage};

pub mod url;
pub mod systems;
pub mod resources;

pub struct Client {
    pub handle: ezsockets::Client<Self>,
    pub tx: Sender<ChatMessage>,
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        tracing::info!("received message: {text}");
        let message = serde_json::from_str::<ChatMessage>(&text)
            .map_err(|e| Error::from(e.to_string()))?;
        self.tx.send(message).map_err(|e| Error::from(e.to_string())) // .await?

    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        tracing::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        println!("recv call");
        match call {
            Call::NewLine(handle, line) => {
                println!("recv newline: {line}");
                tracing::info!("sending {line}");
                let msg = OutMessage {
                    handle,
                    message: line
                };
                self.handle.text(serde_json::to_string(&msg)?);
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum Call {
    NewLine(ChatHandle, String),
}
