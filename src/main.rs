#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  // https://bevy-cheatbook.github.io/platforms/windows.html#disabling-the-windows-console
use std::{env, fmt};
use std::fmt::Formatter;
use std::string::ToString;
use bevy::{prelude::*, render::camera::Projection, window::PrimaryWindow};
use bevy::winit::WinitWindows;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::window::{WindowTheme};
use image;
use crossbeam_channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use ezsockets::ClientConfig;
use ezsockets::Error;
use ezsockets::{Client as EzClient};
use serde::{Serialize, Deserialize};
use bevy::utils::{tracing};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_tokio_tasks::TokioTasksRuntime;
use url::Url;
use serde_json;
use tokio;
use websocket::futures::Future;

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}


use phf::phf_set;

// TODO from backend init
static CHAT_HANDLES: phf::Set<&'static str> = phf_set! {
    "tete",
    "pepe"
};

#[derive(Default, Resource)]
struct UiState {
    chat_input_text: String,
    login_input_text: String,
    sending_chat_message: bool,
    chat_handle: Option<ChatHandle>,
    messages: Vec<ChatMessage>,
}

#[derive(Event)]
struct ChatMessageSentStartedEvent(pub String);

#[derive(Event)]
struct ChatMessageSentSuccessEvent(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatHandle(pub String);
impl fmt::Display for ChatHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessageText(pub String);

impl fmt::Display for ChatMessageText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    message: ChatMessageText,
    handle: ChatHandle,
    // chatId
}

#[derive(Event, Clone)]
struct ChatMessageReceivedEvent(pub ChatMessage);

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

#[derive(Event)]
struct LoggedIn(pub ChatHandle);

#[derive(Resource)]
struct WsUrl(pub String);

use config::Config;

impl Default for WsUrl {
    fn default() -> Self {
        let mut exe_path = env::current_exe().unwrap();
        exe_path.pop(); // remove the file itself
        let p = exe_path.join("assets/Settings");
        let o_path = p.to_str();
        let path = o_path.expect("Settings path not here?");
        let settings = Config::builder()
            .add_source(config::File::with_name(path))
            // .add_source(config::Environment::default())
            .build()
            .unwrap();
        let ws_url = settings.get_string("WS_URL").expect("WS_URL expected at this point");
        WsUrl(ws_url.to_string())
    }
}

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
        }).add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),)
        .add_plugins(EguiPlugin)
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<UiState>()
        .init_resource::<WsUrl>()
        // .init_resource::<StreamReceiver>()
        .add_systems(Startup, set_window_icon)
        .add_systems(Startup, setup_system)
        .add_systems(Startup, websocket_system)
        .add_systems(Update, сhat_ui_system)
        .add_systems(Update, login_ui_system)
        .add_systems(Update, logged_in_system)
        .add_event::<ChatMessageSentStartedEvent>()
        .add_event::<ChatMessageSentSuccessEvent>()
        .add_event::<ChatMessageReceivedEvent>()
        .add_event::<LoggedIn>()
        // .add_systems(Startup, configure_ui_state_system)
        .add_systems(Update, update_camera_transform_system)
        .add_systems(Update, handle_chat_message_sent_started_event_system)
        .add_systems(Update, handle_chat_message_sent_success_event_system)
        .add_systems(Update, handle_chat_message_received_event_system)
        .add_systems(Update, read_stream_system)
        .run();
}

fn login_ui_system(
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

fn logged_in_system(
    mut ui_state: ResMut<UiState>,
    mut event_reader: EventReader<LoggedIn>
) {
    for e in event_reader.iter() {
        ui_state.chat_handle = Some(e.0.clone());
        ui_state.login_input_text = "".to_string();
    }
}

fn сhat_ui_system(
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
    // TODO remove, from cube example
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

fn handle_chat_message_sent_started_event_system(
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


fn handle_chat_message_sent_success_event_system(
    mut chat_message_sent_success_events_r: EventReader<ChatMessageSentSuccessEvent>,
    mut stream_sender: ResMut<WsClient>,
    ui_state: Res<UiState>
) {
    for event in chat_message_sent_success_events_r.iter() {
        println!("sending msg to stream");
        stream_sender.0.call(Call::NewLine(ui_state.chat_handle.clone().expect("chat handle is supposed to be here"), event.0.clone()));
    }
}

fn handle_chat_message_received_event_system(
    mut events: EventReader<ChatMessageReceivedEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in events.iter() {
        ui_state.messages.push(event.0.clone());
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
    // let (icon_rgba, icon_width, icon_height) = {
    //     let image = image::open("assets/icon.png")
    //         .expect("Failed to open icon path")
    //         .into_rgba8();
    //     let (width, height) = image.dimensions();
    //     let rgba = image.into_raw();
    //     (rgba, width, height)
    // };
    //
    // let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    //
    // primary.set_window_icon(Some(icon));
}

#[derive(Debug)]
enum Call {
    NewLine(ChatHandle, String),
}

struct Client {
    handle: ezsockets::Client<Self>,
    tx: Sender<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OutMessage {
    handle: ChatHandle,
    message: String
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

#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<ChatMessage>);

#[derive(Resource, Deref)]
struct StreamSender(Sender<String>);
#[derive(Resource, Deref)]
struct WsClient(EzClient<Client>);

fn websocket_system(mut commands: Commands, runtime: ResMut<TokioTasksRuntime>, ws_url: Res<WsUrl>) {
    let url = ws_url.0.to_string();
    let (tx, rx) = bounded::<ChatMessage>(10);
    let (tx0, rx0) = bounded::<String>(10);
    let (handle, future) = runtime.runtime().block_on(runtime.spawn_background_task(|_ctx| async move {
        println!("This task is running on a background thread");
        // tracing_subscriber::fmt::init();
        let url = Url::parse(&url).unwrap();
        let config = ClientConfig::new(url);
        // TODO handle no network
        ezsockets::connect(|handle| Client { handle, tx }, config).await

    })).unwrap();
    // handle.call(Call::NewLine("hello".to_string()));
    // tokio::spawn(async move {
    //     loop {
    //         for m in rx0.try_iter() {
    //             println!("sending message to handle: {}", m);
    //             handle.call(Call::NewLine(m));
    //         }
    //     }
    // });
    runtime.spawn_background_task(|_ctx| async move {
        future.await.unwrap();
    });

    commands.insert_resource(StreamReceiver(rx));
    commands.insert_resource(StreamSender(tx0));
    commands.insert_resource(WsClient(handle));


}

fn read_stream_system(receiver: Res<StreamReceiver>, mut events: EventWriter<ChatMessageReceivedEvent>) {
    for m in receiver.try_iter() {
        // TODO parse
        events.send(ChatMessageReceivedEvent(m));
    }
}