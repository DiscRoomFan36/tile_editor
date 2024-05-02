mod mouse_stuff;
mod tile_grid;

use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bevy::{asset::LoadedFolder, input, prelude::*};

use mouse_stuff::*;
use tile_grid::*;

const SQUARE_SIZE: f32 = 64.0;
const SQUARE_SPACING: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CameraMousePlugin))
        .init_resource::<MyIconServer>()
        .insert_resource(MainGrid {
            grid: TileGrid::new(6, 4),
        })
        .add_systems(Startup, setup_grid)
        .add_systems(
            PostMouseUpdate,
            (mark_box_clicked, update_clicked_on).chain(),
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            PreUpdate,
            (icon_server_keyboard_handler, grid_save_load_handler),
        )
        .run();
}

#[derive(Resource)]
struct MainGrid {
    grid: TileGrid<usize>,
}

#[derive(Component, Debug, Clone, Copy)]
struct TileMarker {
    pos: (usize, usize),
    // x: usize,
    // y: usize,
}

#[derive(Resource)]
struct MyIconServer {
    selected: usize,
    _asset_folder: Handle<LoadedFolder>, // @Unused: Maybe don't need?
    // _asset_names: Vec<String>,
    assets: Vec<(String, Handle<Image>)>,
}

impl MyIconServer {
    fn get_selected(&self) -> Handle<Image> {
        return self.assets[self.selected].1.clone();
    }

    fn get_by_id(&self, id: usize) -> Handle<Image> {
        assert!(id < self.assets.len(), "Id is within bounds");
        return self.assets[id].1.clone();
    }

    fn get_default(&self) -> Handle<Image> {
        return self.assets[0].1.clone(); // @Cleanup: Canonize default
    }

    fn get_index(&self) -> usize {
        self.selected
    }

    fn increase(&mut self) {
        self.selected = (self.selected + 1) % self.assets.len();
    }

    fn decrease(&mut self) {
        if self.selected == 0 {
            self.selected = self.assets.len()
        }
        self.selected = self.selected - 1;
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

        let handles = names
            .map(|path| (path.clone(), asset_server.load(path)))
            .collect();

        MyIconServer {
            selected: 1,
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
        icon_server.increase()
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        icon_server.decrease()
    }
}

impl ToAndFromJsonValue for usize {
    fn to_json(&self) -> json::JsonValue {
        json::from(*self)
    }

    fn from_json(json: json::JsonValue) -> Option<usize> {
        json.as_usize()
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

        let mut input = File::open(FILE_NAME).expect("File exists");
        let mut buffer = String::new();
        input.read_to_string(&mut buffer).expect("Read to buffer");

        main_grid.grid = TileGrid::from_json(json::parse(&buffer).unwrap()).expect("Grid loaded");

        for (mut handle, tile) in &mut query {
            *handle = if let Some(id) = main_grid.grid.get(tile.pos) {
                icon_server.get_by_id(*id)
            } else {
                icon_server.get_default()
            };
        }
    }
}

#[derive(Component)]
struct GridParentMarker;

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

            let texture = if let Some(id) = main_grid.grid.get((i, j)) {
                icon_server.get_by_id(*id)
            } else {
                icon_server.get_default()
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
fn update_clicked_on(
    mut query: Query<(&mut Handle<Image>, &TileMarker, Entity), With<ClickedOnMarker>>,
    mut commands: Commands,
    icon_server: Res<MyIconServer>,
    mut main_grid: ResMut<MainGrid>,
) {
    for (mut handle, tile_marker, entity) in &mut query {
        main_grid.grid.set(tile_marker.pos, icon_server.get_index());

        *handle = icon_server.get_selected();
        commands.entity(entity).remove::<ClickedOnMarker>();
    }
}
