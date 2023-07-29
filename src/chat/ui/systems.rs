use bevy_egui::{egui, EguiContexts};
use bevy::prelude::{EventReader, EventWriter, ResMut};
use crate::chat::message::events::ChatMessageReceivedEvent;
use crate::chat::ui::events::{ChatMessageSentStartedEvent, ChatMessageSentSuccessEvent};
use crate::login::events::LoggedIn;
use crate::ui::resources::{OccupiedScreenSpace, UiState};

pub fn chat_ui_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut ui_state: ResMut<UiState>,
    mut chat_message_sent_started_events: EventWriter<ChatMessageSentStartedEvent>,
    chat_message_sent_success_events: EventReader<ChatMessageSentSuccessEvent>,
    mut logged_in_events: EventReader<LoggedIn>
) {
    let chat_handle = ui_state.chat_handle.clone();
    if chat_handle.is_none() {
        return;
    }
    let ctx = contexts.ctx_mut();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            // add a list of hardcoded one user
            ui.separator();
            ui.label("You");
            ui.separator();
            ui.set_min_width(150.0);
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    let just_logged_in = logged_in_events.iter().len() > 0;
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            if ui_state.sending_chat_message {
                ui.label("Sending message...");
            } else {
                let input = ui.horizontal(|ui| {
                    let res = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut ui_state.chat_input_text));
                    if !chat_message_sent_success_events.is_empty() {
                        res.request_focus();
                    }
                    if just_logged_in {
                        res.request_focus();
                    }
                    res
                });

                handle_chat_input(&input.response.ctx, &mut ui_state, &mut chat_message_sent_started_events);

            }
            ui.set_min_height(100.0);
        })
        .response
        .rect
        .height();
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let messages = &ui_state.messages;
            for (i, m) in messages.iter().enumerate() {
                let res = ui.vertical(|ui| {
                    ui.label(format!("{}: {}", m.handle, m.message));
                });
                // Add separator between messages
                if i < messages.len() - 1 {
                    ui.separator();
                } else {
                    // scroll_to_me
                    res.response.scroll_to_me(Some(egui::Align::BOTTOM));
                }
            }
        });
        ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    });
}

fn handle_chat_input(
    ctx: &egui::Context,
    ui_state: &mut ResMut<UiState>,
    events: &mut EventWriter<ChatMessageSentStartedEvent>,
) {
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        if ui_state.chat_input_text.len() > 0 {
            ui_state.sending_chat_message = true;

            events.send(ChatMessageSentStartedEvent(ui_state.chat_input_text.clone()));
            ui_state.chat_input_text.clear();
        }
    }
}

pub fn handle_chat_message_sent_started_event_system(
    mut ui_state: ResMut<UiState>,
    mut chat_message_sent_started_events: EventReader<ChatMessageSentStartedEvent>,
    mut chat_message_sent_success_events: EventWriter<ChatMessageSentSuccessEvent>,

) {
    for event in chat_message_sent_started_events.iter() {
        println!("sending message from input: {}", event.0);
        ui_state.sending_chat_message = false;
        chat_message_sent_success_events.send(ChatMessageSentSuccessEvent(event.0.clone()));
    }
}

pub fn handle_chat_message_received_event_system(
    mut events: EventReader<ChatMessageReceivedEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in events.iter() {
        ui_state.messages.push(event.0.clone());
    }
}
