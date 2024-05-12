mod mouse_stuff;
mod tile_grid;

use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bevy::{asset::LoadedFolder, input, prelude::*};

use mouse_stuff::*;
use tile_grid::*;

// TODO? Some more window management in the future
const WINDOW_SIZE: (f32, f32) = (1280.0, 720.0);

// TODO: be able to change these at run time.
const SQUARE_SIZE: f32 = 64.0;
const SQUARE_SPACING: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tile Editor".to_string(),
                    resolution: WINDOW_SIZE.into(),
                    ..default()
                }),
                ..default()
            }),
            CameraMousePlugin,
        ))
        .init_resource::<MyIconServer>()
        .insert_resource(MainGrid {
            grid: TileGrid::new(6, 4),
        })
        .add_systems(Startup, (setup_grid, setup_pallet))
        .add_systems(
            PreUpdate,
            (icon_server_keyboard_handler, grid_save_load_handler),
        )
        .add_systems(
            PostMouseUpdate,
            (
                mark_box_clicked,
                (update_clicked_on_tile, update_clicked_on_pallet),
            )
                .chain(),
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, update_pallet_highlights)
        .run();
}

#[derive(Resource)]
struct MainGrid {
    grid: TileGrid<String>, // TODO? Use str
}

#[derive(Component, Debug, Clone, Copy)]
struct TileMarker {
    pos: (usize, usize),
}

#[derive(Resource)]
struct MyIconServer {
    selected: String,                    // TODO: use str
    _asset_folder: Handle<LoadedFolder>, // @Unused: Maybe don't need?
    assets: Vec<(String, Handle<Image>)>,
}

impl MyIconServer {
    fn get_selected_handle(&self) -> Handle<Image> {
        return self
            .assets
            .iter()
            .find(|(name, _)| *name == self.selected)
            .map(|(_, handle)| handle.clone())
            .expect("self.selected is valid");
    }

    fn get_selected_name(&self) -> &str {
        return &self.selected;
    }

    fn get_by_name(&self, name: &str) -> Handle<Image> {
        self.assets
            .iter()
            .find(|(asset_name, _)| name == asset_name)
            .map(|(_, handle)| handle.clone())
            .expect("Name exists in assets")
    }

    fn get_default_handle(&self) -> Handle<Image> {
        return self.assets[0].1.clone(); // @Cleanup: Canonize default
    }

    fn set_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.selected = name.to_owned();
    }

    fn cycle_forwards(&mut self) {
        let mut index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == self.selected)
            .map(|(i, _)| i)
            .unwrap();

        index = (index + 1) % self.assets.len();

        self.selected = self
            .assets
            .get(index)
            .map(|(name, _)| name.to_owned())
            .unwrap();
    }

    fn cycle_backwards(&mut self) {
        let mut index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (name, _))| *name == self.selected)
            .map(|(i, _)| i)
            .unwrap();

        if index == 0 {
            index = self.assets.len()
        }
        index = index - 1;

        self.selected = self
            .assets
            .get(index)
            .map(|(name, _)| name.to_owned())
            .unwrap();
    }
}

impl FromWorld for MyIconServer {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let icons = asset_server.load_folder("icons"); // loads all files in parallel

        // hmmm... might need a better way to do this
        let paths = fs::read_dir("./assets/icons").expect("Icon folder exists");

        let names = paths
            .map(|path| {
                path.unwrap()
                    .path()
                    .file_name()
                    .and_then(|p| p.to_str())
                    .and_then(|p| Some(p.to_string()))
                    .unwrap()
            })
            .map(|str| format!("icons/{str}"));

        let handles: Vec<(String, Handle<Image>)> = names
            .map(|path| (path.clone(), asset_server.load(path)))
            .collect();

        MyIconServer {
            selected: handles
                .get(1)
                .map(|(name, _)| name.to_owned())
                .expect("More than one element in pallet at startup"),
            _asset_folder: icons,
            assets: handles,
        }
    }
}

fn icon_server_keyboard_handler(
    keys: Res<input::ButtonInput<KeyCode>>,
    mut icon_server: ResMut<MyIconServer>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        icon_server.cycle_forwards()
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        icon_server.cycle_backwards()
    }
}

impl ToAndFromJsonValue for String {
    fn to_json(&self) -> json::JsonValue {
        json::from(self.to_owned())
    }

    fn from_json(json: json::JsonValue) -> Option<Self> {
        json.as_str().and_then(|str| Some(str.to_owned()))
    }
}

fn grid_save_load_handler(
    keys: Res<input::ButtonInput<KeyCode>>,
    mut main_grid: ResMut<MainGrid>,
    mut query: Query<(&mut Handle<Image>, &TileMarker)>,
    icon_server: Res<MyIconServer>,
) {
    const FILE_NAME: &str = "quick-save.json";
    if keys.just_pressed(KeyCode::KeyP) {
        info!("Saving Grid!");

        let json_string = main_grid.grid.to_json().to_string();
        let mut output = File::create(FILE_NAME).expect("File was created");
        write!(output, "{}", json_string).expect("Write to file");
    }

    if keys.just_pressed(KeyCode::KeyL) {
        info!("Loading Saved Grid!");

        let Ok(mut input) = File::open(FILE_NAME) else {
            info!("No quick save file");
            return;
        };
        let mut buffer = String::new();
        input.read_to_string(&mut buffer).expect("Read to buffer");

        main_grid.grid = TileGrid::from_json(json::parse(&buffer).unwrap()).expect("Grid loaded");

        for (mut handle, tile) in &mut query {
            *handle = if let Some(name) = main_grid.grid.get(tile.pos) {
                icon_server.get_by_name(name)
            } else {
                icon_server.get_default_handle()
            };
        }
    }
}

fn setup_grid(main_grid: Res<MainGrid>, mut commands: Commands, icon_server: Res<MyIconServer>) {
    const SCALED_SQUARE: f32 = SQUARE_SIZE / 32.0; // div by 32 because thats how many pixels wide the image is
    const TEXTURE_SCALE: Vec3 = Vec3::new(SCALED_SQUARE, SCALED_SQUARE, 1.0);

    let (n, m) = main_grid.grid.get_size();

    let grid_width = SQUARE_SIZE * n as f32 + SQUARE_SPACING * n as f32;
    let grid_hight = SQUARE_SIZE * m as f32 + SQUARE_SPACING * m as f32;

    for j in 0..m {
        for i in 0..n {
            // Color probably isn't right for what i wanted but it changes there color as well as can be expected
            let color = Color::hsl(360.0 * (i + j * m) as f32 / (n * m) as f32, 0.95, 0.7);

            let transform = Transform {
                translation: Vec3::new(
                    (-grid_width / 2.0) + (i as f32 / n as f32 * grid_width),
                    (-grid_hight / 2.0) + (j as f32 / m as f32 * grid_hight),
                    0.0,
                ),
                scale: TEXTURE_SCALE,
                ..default()
            };

            let texture = if let Some(name) = main_grid.grid.get((i, j)) {
                icon_server.get_by_name(name)
            } else {
                icon_server.get_default_handle()
            };

            commands.spawn((
                SpriteBundle {
                    texture,
                    transform,
                    sprite: Sprite { color, ..default() },
                    ..default()
                },
                TileMarker { pos: (i, j) },
                MouseCollider(transform),
            ));
        }
    }
}

#[derive(Component)]
struct PalletMarker {
    name: String,
}

#[derive(Component)]
struct ColorHolder {
    base_color: Color,
    highlight_color: Color,
}

fn setup_pallet(icon_server: Res<MyIconServer>, mut commands: Commands) {
    // TODO: Copy pasta, get a better system for this.
    const SCALED_SQUARE: f32 = SQUARE_SIZE / 32.0; // div by 32 because thats how many pixels wide the image is
    const TEXTURE_SCALE: Vec3 = Vec3::new(SCALED_SQUARE, SCALED_SQUARE, 1.0);

    // TODO: Set this based on texture scale?
    const PALLET_COLUMNS: usize = 3;

    const PADDING: f32 = 10.0;

    // TODO? setup_grid should use this method as well?
    let start_x = (-WINDOW_SIZE.0 / 2.0) + (SQUARE_SIZE / 2.0) + PADDING;
    let start_y = (WINDOW_SIZE.1 / 2.0) - (SQUARE_SIZE / 2.0) - PADDING;

    let assets = &icon_server.assets;

    for (i, (name, asset)) in assets.iter().enumerate() {
        let (d, m) = ((i / PALLET_COLUMNS) as f32, (i % PALLET_COLUMNS) as f32);
        let transform = Transform {
            translation: Vec3::new(
                start_x + (m * (SQUARE_SIZE + SQUARE_SPACING)),
                start_y - (d * (SQUARE_SIZE + SQUARE_SPACING)),
                1.0,
            ),
            scale: TEXTURE_SCALE,
            ..default()
        };
        let base_color = Color::hsl(150.0, 0.7, 0.9);
        let highlight_color = Color::hsl(50.0, 1.0, 0.55);

        commands.spawn((
            SpriteBundle {
                texture: asset.clone(),
                transform,
                sprite: Sprite {
                    color: base_color,
                    ..default()
                },
                ..default()
            },
            PalletMarker { name: name.clone() },
            MouseCollider(transform),
            ColorHolder {
                base_color,
                highlight_color,
            },
        ));
    }
}

fn update_pallet_highlights(
    mut query: Query<(&PalletMarker, &ColorHolder, &mut Sprite)>,
    icon_server: Res<MyIconServer>,
) {
    for (PalletMarker { name }, color_holder, mut sprite) in &mut query {
        if icon_server.get_selected_name() == name {
            sprite.color = color_holder.highlight_color;
        } else {
            sprite.color = color_holder.base_color;
        }
    }
}

#[derive(Component)]
pub struct ClickedOnMarker;

fn mark_box_clicked(
    mut ev_mouse_collision: EventReader<MouseCollisionEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    for MouseCollisionEvent(entity) in ev_mouse_collision.read() {
        if buttons.just_pressed(MouseButton::Left) {
            commands.entity(*entity).insert(ClickedOnMarker);
        }
    }
}
fn update_clicked_on_tile(
    mut query: Query<(&mut Handle<Image>, &TileMarker, Entity), With<ClickedOnMarker>>,
    mut commands: Commands,
    icon_server: Res<MyIconServer>,
    mut main_grid: ResMut<MainGrid>,
) {
    for (mut handle, tile_marker, entity) in &mut query {
        main_grid
            .grid
            .set(tile_marker.pos, icon_server.get_selected_name().to_owned());

        *handle = icon_server.get_selected_handle();
        commands.entity(entity).remove::<ClickedOnMarker>();
    }
}

fn update_clicked_on_pallet(
    mut query: Query<(&PalletMarker, Entity), With<ClickedOnMarker>>,
    mut commands: Commands,
    mut icon_server: ResMut<MyIconServer>,
) {
    for (PalletMarker { name }, entity) in &mut query {
        info!("Setting selected to {name}");
        icon_server.set_by_name(name);

        commands.entity(entity).remove::<ClickedOnMarker>();
    }
}
