use std::fs;

use bevy::{input, prelude::*};

pub struct IconServerPlugin;
impl Plugin for IconServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyIconServer>()
            .add_systems(PreUpdate, icon_server_cycle_handles_handler);
    }
}

#[derive(Resource)]
pub struct MyIconServer {
    pub assets: Vec<(String, Handle<Image>)>,
    selected: String,     // TODO: use str
    default_icon: String, // TODO: use str // @Think about weather this should be in the json
}

impl MyIconServer {
    pub fn get_selected_name(&self) -> &str {
        return &self.selected;
    }

    pub fn get_default_name(&self) -> &str {
        return &self.default_icon;
    }

    pub fn get_by_name(&self, name: &str) -> Option<Handle<Image>> {
        self.assets
            .iter()
            .find(|(asset_name, _)| name == asset_name)
            .map(|(_, handle)| handle.clone())
    }

    pub fn _get_selected_handle(&self) -> Handle<Image> {
        self.get_by_name(&self.selected)
            .expect("self.selected is valid")
    }

    pub fn get_default_handle(&self) -> Handle<Image> {
        self.get_by_name(&self.default_icon)
            .expect("self.default_icon is valid")
    }

    pub fn set_selected_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.selected = name.to_owned();
    }

    pub fn set_default_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.default_icon = name.to_owned();
    }

    pub fn next_icon_name_in_cycle(&self, name: &str) -> &str {
        let index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (asset_name, _))| asset_name == name)
            .map(|(i, _)| i)
            .unwrap();

        self.assets
            .get((index + 1) % self.assets.len())
            .map(|(name, _)| name)
            .unwrap()
    }

    pub fn prev_icon_name_in_cycle(&self, name: &str) -> &str {
        let mut index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (asset_name, _))| asset_name == name)
            .map(|(i, _)| i)
            .unwrap();

        if index == 0 {
            index = self.assets.len()
        }
        index = index - 1;

        self.assets.get(index).map(|(name, _)| name).unwrap()
    }
}

// TODO: this is probably broken for paths that are not in the assets folder
fn load_folder(path: &str, asset_server: &AssetServer) -> Vec<(String, Handle<Image>)> {
    // path is like "./assets/icons"
    let paths = fs::read_dir(path).expect("Valid directory");

    let last_path_name = path.split("/").last().unwrap();

    let names = paths
        .map(|path| path.unwrap())
        .map(|path| {
            path.path()
                .file_name()
                .and_then(|p| p.to_str())
                .and_then(|p| Some(p.to_string()))
                .unwrap()
        })
        .map(|str| format!("{last_path_name}/{str}"));

    let handles: Vec<(String, Handle<Image>)> = names
        .map(|path| (path.clone(), asset_server.load(path)))
        .collect();

    return handles;
}

impl FromWorld for MyIconServer {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let handles = load_folder("./assets/icons", asset_server);

        MyIconServer {
            selected: handles
                .get(1)
                .map(|(name, _)| name.to_owned())
                .expect("More than one element in pallet at startup"),
            default_icon: handles
                .get(0)
                .map(|(name, _)| name.to_owned())
                .expect("More than zero elements in pallet at startup"),
            assets: handles,
        }
    }
}

pub fn icon_server_cycle_handles_handler(
    keys: Res<input::ButtonInput<KeyCode>>,
    mut icon_server: ResMut<MyIconServer>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        icon_server.selected = icon_server
            .next_icon_name_in_cycle(&icon_server.selected)
            .to_owned()
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        icon_server.selected = icon_server
            .prev_icon_name_in_cycle(&icon_server.selected)
            .to_owned()
    }

    if keys.just_pressed(KeyCode::KeyX) {
        icon_server.default_icon = icon_server
            .next_icon_name_in_cycle(&icon_server.default_icon)
            .to_owned()
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        icon_server.default_icon = icon_server
            .prev_icon_name_in_cycle(&icon_server.default_icon)
            .to_owned()
    }
}
