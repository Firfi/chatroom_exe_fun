use bevy::prelude::{Deref, Resource};
use ezsockets::Client as EzClient;
use crate::ws::Client;

#[derive(Resource, Deref)]
pub struct WsClient(pub EzClient<Client>);
