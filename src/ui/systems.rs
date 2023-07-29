use bevy::prelude::{Entity, NonSend, Query, With};
use bevy::winit::WinitWindows;
use bevy::window::PrimaryWindow;
use winit::window::Icon;
use crate::asset::utils::exe_asset_path;

// ostensibly sets an icon
pub fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    // bevy window
) {
    let primary = windows.get_window(primary_window.single()).unwrap();

    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let path = exe_asset_path("assets/icon.png".to_string());
        let image = image::open(path.as_str())
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}
