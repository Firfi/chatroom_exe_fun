use bevy_egui::{egui, EguiContexts};
use bevy::prelude::{EventReader, EventWriter, ResMut};
use crate::chat::{CHAT_HANDLES, ChatHandle};
use crate::login::events::LoggedIn;
use crate::ui::resources::UiState;

pub fn login_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut event_writer: EventWriter<LoggedIn>
) {
    if ui_state.chat_handle.is_some() {
        return;
    }
    let ctx = contexts.ctx_mut();
    egui::Window::new("Login").auto_sized().show(ctx, |ui| {
        let res = ui.add(egui::TextEdit::singleline(&mut ui_state.login_input_text));
        res.request_focus();
        if res.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            match CHAT_HANDLES.iter().find(|h| h.to_string() == ui_state.login_input_text) {
                Some(h) => {
                    event_writer.send(LoggedIn(ChatHandle(h.to_string())));
                }
                None => {

                }
            }
        }
    });
}

pub fn logged_in_system(
    mut ui_state: ResMut<UiState>,
    mut event_reader: EventReader<LoggedIn>
) {
    for e in event_reader.iter() {
        ui_state.chat_handle = Some(e.0.clone());
        ui_state.login_input_text = "".to_string();
    }
}
