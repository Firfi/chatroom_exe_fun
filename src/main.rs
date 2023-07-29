// https://bevy-cheatbook.github.io/platforms/windows.html#disabling-the-windows-console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod asset;
mod ws;
mod chat;
mod login;
mod ui;
mod misc;
use std::string::ToString;
use bevy::{prelude::*};
use bevy_egui::EguiPlugin;
use bevy::window::WindowTheme;use bevy_embedded_assets::EmbeddedAssetPlugin;
use chat::message::events::ChatMessageReceivedEvent;
use login::events::LoggedIn;
use crate::ws::url::resources::WsUrl;
use chat::ui::events::{ChatMessageSentStartedEvent, ChatMessageSentSuccessEvent};
use ui::resources::{OccupiedScreenSpace, UiState};
use crate::chat::message::systems::handle_chat_message_sent_success_event_system;
use crate::chat::systems::read_stream_system;
use crate::chat::ui::systems::{handle_chat_message_received_event_system, handle_chat_message_sent_started_event_system, chat_ui_system};
use crate::login::systems::{logged_in_system, login_ui_system};
use crate::misc::systems::{cube_system, update_camera_transform_system};
use crate::ui::systems::set_window_icon;
use crate::ws::systems::websocket_system;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FUN chatroom!".to_string(),
                //resolution: (500., 300.).into(),
                //present_mode: PresentMode::AutoVsync,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }).add_before::<AssetPlugin, _>(EmbeddedAssetPlugin),)
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<UiState>()
        .init_resource::<WsUrl>()
        .add_systems(Startup, set_window_icon)
        .add_systems(Startup, cube_system)
        .add_systems(Startup, websocket_system)
        .add_systems(Update, chat_ui_system)
        .add_systems(Update, login_ui_system)
        .add_systems(Update, logged_in_system)
        .add_event::<ChatMessageSentStartedEvent>()
        .add_event::<ChatMessageSentSuccessEvent>()
        .add_event::<ChatMessageReceivedEvent>()
        .add_event::<LoggedIn>()
        .add_systems(Update, update_camera_transform_system)
        .add_systems(Update, handle_chat_message_sent_started_event_system)
        .add_systems(Update, handle_chat_message_sent_success_event_system)
        .add_systems(Update, handle_chat_message_received_event_system)
        .add_systems(Update, read_stream_system)
        .run();
}