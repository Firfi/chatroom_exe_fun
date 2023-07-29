use bevy::prelude::{EventReader, Res, ResMut};
use crate::chat::ui::events::ChatMessageSentSuccessEvent;
use crate::ui::resources::UiState;
use crate::ws::Call;
use crate::ws::resources::WsClient;

pub fn handle_chat_message_sent_success_event_system(
    mut chat_message_sent_success_events_r: EventReader<ChatMessageSentSuccessEvent>,
    ws_client: ResMut<WsClient>,
    ui_state: Res<UiState>
) {
    for event in chat_message_sent_success_events_r.iter() {
        println!("sending msg to stream");
        ws_client.0.call(Call::NewLine(ui_state.chat_handle.clone().expect("chat handle is supposed to be here"), event.0.clone()));
    }
}
