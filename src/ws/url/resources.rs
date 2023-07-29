use bevy::prelude::Resource;
use config::Config;
use crate::asset::utils::exe_asset_path;

#[derive(Resource)]
pub struct WsUrl(pub String);

impl Default for WsUrl {
    fn default() -> Self {
        let path = exe_asset_path("assets/Settings".to_string());
        let settings = Config::builder()
            .add_source(config::File::with_name(path.as_str()))
            // .add_source(config::Environment::default())
            .build()
            .unwrap();
        let ws_url = settings.get_string("WS_URL").expect("WS_URL expected at this point");
        WsUrl(ws_url.to_string())
    }
}