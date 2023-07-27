#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  // https://bevy-cheatbook.github.io/platforms/windows.html#disabling-the-windows-console

use bevy::{prelude::*, render::camera::Projection, window::PrimaryWindow};
use bevy::winit::WinitWindows;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::window::{WindowTheme};
use winit::window::{Icon};
use image;
use std::sync::mpsc::channel;
use std::thread;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Default, Resource)]
struct UiState {
    chat_input_text: String,
    sending_chat_message: bool,
    messages: Vec<(String, String)>,
}

#[derive(Event)]
struct ChatMessageSentStartedEvent(pub String);

#[derive(Event)]
struct ChatMessageSentSuccessEvent(pub String);

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FUN chatroom!".into(),
                //resolution: (500., 300.).into(),
                //present_mode: PresentMode::AutoVsync,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }),)
        .add_plugins(EguiPlugin)
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<UiState>()
        .add_systems(Startup, set_window_icon)
        .add_systems(Startup, setup_system)
        .add_systems(Startup, websocket_system)
        .add_systems(Update, ui_example_system)
        .add_event::<ChatMessageSentStartedEvent>()
        .add_event::<ChatMessageSentSuccessEvent>()
        // .add_systems(Startup, configure_ui_state_system)
        .add_systems(Update, update_camera_transform_system)
        .add_systems(Update, handle_chat_message_sent_started_event_system)
        .add_systems(Update, handle_chat_message_sent_success_event_system)
        .run();
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut ui_state: ResMut<UiState>,
    mut chat_message_sent_started_events: EventWriter<ChatMessageSentStartedEvent>,
    chat_message_sent_success_events: EventReader<ChatMessageSentSuccessEvent>,
) {
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
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            if ui_state.sending_chat_message {
                ui.label("Sending message...");
            } else {
                ui.horizontal(|ui| {
                    let res = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut ui_state.chat_input_text));
                    if !chat_message_sent_success_events.is_empty() {
                        res.request_focus();
                    }
                });

                handle_chat_input(&ui, &mut ui_state, &mut chat_message_sent_started_events);

            }
            ui.set_min_height(100.0);
        })
        .response
        .rect
        .height();
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let messages = &ui_state.messages;
            for (i, (name, message)) in messages.iter().enumerate() {
                let res = ui.vertical(|ui| {
                    ui.label(format!("{}: {}", name, message));
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

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let camera_pos = Vec3::new(-2.0, 2.5, 5.0);
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);
    commands.insert_resource(OriginalCameraTransform(camera_transform));

    commands.spawn(Camera3dBundle {
        transform: camera_transform,
        ..Default::default()
    });
}

fn update_camera_transform_system(
    occupied_screen_space: Res<OccupiedScreenSpace>,
    original_camera_transform: Res<OriginalCameraTransform>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Projection, &mut Transform)>,
) {
    let (camera_projection, mut transform) = match camera_query.get_single_mut() {
        Ok((Projection::Perspective(projection), transform)) => (projection, transform),
        _ => unreachable!(),
    };

    let distance_to_target = (CAMERA_TARGET - original_camera_transform.translation).length();
    let frustum_height = 2.0 * distance_to_target * (camera_projection.fov * 0.5).tan();
    let frustum_width = frustum_height * camera_projection.aspect_ratio;

    let window = windows.single();

    let left_taken = occupied_screen_space.left / window.width();
    let right_taken = occupied_screen_space.right / window.width();
    let top_taken = occupied_screen_space.top / window.height();
    let bottom_taken = occupied_screen_space.bottom / window.height();
    transform.translation = original_camera_transform.translation
        + transform.rotation.mul_vec3(Vec3::new(
        (right_taken - left_taken) * frustum_width * 0.5,
        (top_taken - bottom_taken) * frustum_height * 0.5,
        0.0,
    ));
}

fn handle_chat_input(
    ui: &egui::Ui,
    mut ui_state: &mut ResMut<UiState>,
    events: &mut EventWriter<ChatMessageSentStartedEvent>,
) {
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if ui_state.chat_input_text.len() > 0 {
            ui_state.sending_chat_message = true;

            events.send(ChatMessageSentStartedEvent(ui_state.chat_input_text.clone()));
            ui_state.chat_input_text.clear();
        }
    }
}

fn handle_chat_message_sent_started_event_system(
    mut ui_state: ResMut<UiState>,
    mut chat_message_sent_started_events: EventReader<ChatMessageSentStartedEvent>,
    mut chat_message_sent_success_events: EventWriter<ChatMessageSentSuccessEvent>,

) {
    for event in chat_message_sent_started_events.iter() {
        println!("sending message: {}", event.0);
        ui_state.sending_chat_message = false;
        chat_message_sent_success_events.send(ChatMessageSentSuccessEvent(event.0.clone()));
    }
}

fn handle_chat_message_sent_success_event_system(
    mut chat_message_sent_success_events_r: EventReader<ChatMessageSentSuccessEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in chat_message_sent_success_events_r.iter() {
        ui_state.messages.push(("You".to_string(), event.0.clone()));
    }
}

// ostensibly sets an icon
fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    // bevy window
) {
    let primary = windows.get_window(primary_window.single()).unwrap();

    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}

const CONNECTION: &'static str = "ws://localhost:80";

fn websocket_system(events: &mut EventWriter<ChatMessageSentStartedEvent>,) {
    println!("Connecting to {}", CONNECTION);

    // TODO reconnects
    let client = ClientBuilder::new(CONNECTION)
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure()
        .unwrap();

    println!("Successfully connected");

    let (mut receiver, mut sender) = client.split().unwrap();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(OwnedMessage::Close(None));
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    // Got a close message, so send a close message and return
                    let _ = tx_1.send(OwnedMessage::Close(None));
                    return;
                }
                OwnedMessage::Ping(data) => {
                    match tx_1.send(OwnedMessage::Pong(data)) {
                        // Send a pong in response
                        Ok(()) => (),
                        Err(e) => {
                            println!("Receive Loop: {:?}", e);
                            return;
                        }
                    }
                }
                // Say what we received
                _ => {
                    println!("Receive Loop: {:?}", message);
                    events.send(ChatMessageReceivedEvent(message.to_string()));
                },
            }
        }
    });

}