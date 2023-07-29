use bevy::prelude::{Commands, Res, ResMut};
use bevy_tokio_tasks::TokioTasksRuntime;
use crossbeam_channel::bounded;
use ezsockets::ClientConfig;
use url::Url;
use crate::chat::message::ChatMessage;
use crate::chat::resources::{StreamReceiver};
use crate::ws::Client;
use crate::ws::resources::WsClient;
use crate::ws::url::resources::WsUrl;

pub fn websocket_system(mut commands: Commands, runtime: ResMut<TokioTasksRuntime>, ws_url: Res<WsUrl>) {
    let url = ws_url.0.to_string();
    let (tx, rx) = bounded::<ChatMessage>(10);
    let (handle, future) = runtime.runtime().block_on(runtime.spawn_background_task(|_ctx| async move {
        println!("This task is running on a background thread");
        let url = Url::parse(&url).unwrap();
        let config = ClientConfig::new(url);
        // TODO handle no network
        ezsockets::connect(|handle| Client { handle, tx }, config).await

    })).unwrap();
    runtime.spawn_background_task(|_ctx| async move {
        future.await.unwrap();
    });

    commands.insert_resource(StreamReceiver(rx));
    commands.insert_resource(WsClient(handle));


}