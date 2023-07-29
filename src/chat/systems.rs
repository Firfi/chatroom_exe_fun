use bevy::prelude::{EventWriter, Res};
use crate::chat::message::events::ChatMessageReceivedEvent;
use crate::chat::resources::StreamReceiver;

pub fn read_stream_system(receiver: Res<StreamReceiver>, mut events: EventWriter<ChatMessageReceivedEvent>) {
    for m in receiver.try_iter() {
        events.send(ChatMessageReceivedEvent(m));
    }
}
