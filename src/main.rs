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
            grid: TileGrid::new(4, 6),
            old_grid: TileGrid::new(0, 0),
        })
        .add_systems(Startup, setup_pallet)
        .add_systems(
            PreUpdate,
            (
                icon_server_keyboard_handler,
                grid_save_load_handler,
                grid_change_default_handle,
                grid_change_size,
            ),
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
        .add_systems(Update, (update_pallet_highlights, grid_refresh_handler))
        .run();
}

#[derive(Resource)]
struct MainGrid {
    grid: TileGrid<String>,     // TODO? Use str
    old_grid: TileGrid<String>, // TODO? Use str
}

#[derive(Component, Debug, Clone, Copy)]
struct TileMarker {
    pos: (usize, usize),
}

#[derive(Resource)]
struct MyIconServer {
    _asset_folder: Handle<LoadedFolder>, // @Unused: Maybe don't need?
    assets: Vec<(String, Handle<Image>)>,
    selected: String,     // TODO: use str
    default_icon: String, // TODO: use str // TODO: think about weather this should be in the json
}

impl MyIconServer {
    fn get_selected_name(&self) -> &str {
        return &self.selected;
    }

    fn _get_default_name(&self) -> &str {
        return &self.default_icon;
    }

    fn get_by_name(&self, name: &str) -> Option<Handle<Image>> {
        self.assets
            .iter()
            .find(|(asset_name, _)| name == asset_name)
            .map(|(_, handle)| handle.clone())
    }

    fn _get_selected_handle(&self) -> Handle<Image> {
        self.get_by_name(&self.selected)
            .expect("self.selected is valid")
    }

    fn get_default_handle(&self) -> Handle<Image> {
        self.get_by_name(&self.default_icon)
            .expect("self.default_icon is valid")
    }

    fn set_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.selected = name.to_owned();
    }

    fn next_icon_name_in_cycle(&self, name: &str) -> &str {
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

    fn prev_icon_name_in_cycle(&self, name: &str) -> &str {
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
            default_icon: handles
                .get(0)
                .map(|(name, _)| name.to_owned())
                .expect("More than zero elements in pallet at startup"),
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
        icon_server.selected = icon_server
            .next_icon_name_in_cycle(&icon_server.selected)
            .to_owned()
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        icon_server.selected = icon_server
            .prev_icon_name_in_cycle(&icon_server.selected)
            .to_owned()
    }
}

// fn get_grid_positions(size: (usize, usize), start: Vec2, scale: Vec3) -> Vec<((usize, usize), Vec3)> {
fn get_grid_positions(
    size: (usize, usize),
    start: Vec2,
    xy_scaling: Vec2,
    scale: Vec3,
) -> Vec<((usize, usize), Transform)> {
    let (rows, cols) = size;

    let mut transforms = vec![];
    for y in 0..rows {
        for x in 0..cols {
            let transform = Transform {
                translation: Vec3 {
                    x: start.x + (x as f32 * (SQUARE_SIZE + SQUARE_SPACING)) * xy_scaling.x,
                    y: start.y + (y as f32 * (SQUARE_SIZE + SQUARE_SPACING)) * xy_scaling.y,
                    z: 1.0,
                },
                scale,
                ..default()
            };

            transforms.push(((x, y), transform));
        }
    }

    transforms
}

#[derive(Component)]
struct TextMarker;

fn grid_refresh_handler(
    mut main_grid: ResMut<MainGrid>,
    icon_server: Res<MyIconServer>,
    mut query: Query<(&mut Handle<Image>, &TileMarker, Entity)>,
    query_text: Query<Entity, With<TextMarker>>,
    mut commands: Commands,
) {
    if main_grid.grid.size() != main_grid.old_grid.size() {
        for (mut _handle, _tile, entity) in &mut query {
            commands.entity(entity).despawn_recursive(); // despawn children if we do that
        }
        for entity in &query_text {
            commands.entity(entity).despawn();
        }

        let (rows, cols) = main_grid.grid.size();

        let start = Vec2 {
            x: (((cols - 1) / 2) as f32 * -SQUARE_SPACING)
                + ((cols - 1) as f32 / 2.0 * -SQUARE_SIZE),
            y: (((rows - 1) / 2) as f32 * -SQUARE_SPACING)
                + ((rows - 1) as f32 / 2.0 * -SQUARE_SIZE),
        };

        const SCALED_SQUARE: f32 = SQUARE_SIZE / 32.0; // div by 32 because thats how many pixels wide the image is
        const TEXTURE_SCALE: Vec3 = Vec3::new(SCALED_SQUARE, SCALED_SQUARE, 1.0);

        let transforms =
            get_grid_positions((rows, cols), start, Vec2 { x: 1.0, y: 1.0 }, TEXTURE_SCALE);

        for ((x, y), transform) in transforms {
            let color = Color::hsl(
                360.0 * (x + y * cols) as f32 / (rows * cols) as f32,
                0.95,
                0.7,
            );

            let texture = if let Some(name) = main_grid.grid.get((x, y)) {
                icon_server.get_by_name(name).expect("grid is valid")
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
                TileMarker { pos: (x, y) },
                MouseCollider(transform),
            ));
        }

        main_grid.old_grid = main_grid.grid.clone(); // @Think: be smarter? can you be smarter here? would i even be faster?

        commands.spawn((
            Text2dBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: start.x - SQUARE_SIZE,
                        y: start.y - (SQUARE_SIZE / 1.8),
                        z: 2.0,
                    },
                    ..default()
                },
                text: Text::from_section(
                    "(0, 0)",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::GOLD,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Center),
                ..default()
            },
            TextMarker,
        ));

        return; // @think: be smarter? combine the thing below with this?
    }

    for (mut handle, tile, _entity) in &mut query {
        let pos = tile.pos;

        let old_out_of_date = main_grid.old_grid.get(pos) != main_grid.grid.get(pos);
        let default_changed =
            main_grid.grid.get(pos).is_none() && (*handle != icon_server.get_default_handle());

        // compiler should be smart here, hopefully
        if old_out_of_date || default_changed {
            // info!("Updating sprite!");

            *handle = if let Some(name) = main_grid.grid.get(tile.pos) {
                icon_server.get_by_name(name).expect("Tile grid is valid")
            } else {
                icon_server.get_default_handle()
            };

            if old_out_of_date {
                let new_tile = main_grid.grid.get(pos).clone();
                main_grid.old_grid.set(pos, new_tile);
            }
        }
    }
}

// @Think: this is pretty similar to the refresh method... maybe theres something you could do there?
fn setup_pallet(icon_server: Res<MyIconServer>, mut commands: Commands) {
    // TODO: Copy pasta, get a better system for this.
    const SCALED_SQUARE: f32 = SQUARE_SIZE / 32.0; // div by 32 because thats how many pixels wide the image is
    const TEXTURE_SCALE: Vec3 = Vec3::new(SCALED_SQUARE, SCALED_SQUARE, 1.0);

    // TODO: Set this based on texture scale?
    const PALLET_COLUMNS: usize = 3;

    const PADDING: f32 = 10.0;

    let start = Vec2 {
        x: (-WINDOW_SIZE.0 / 2.0) + (SQUARE_SIZE / 2.0) + PADDING,
        y: (WINDOW_SIZE.1 / 2.0) - (SQUARE_SIZE / 2.0) - PADDING,
    };

    let assets = &icon_server.assets;
    let transforms = get_grid_positions(
        (assets.len().div_ceil(PALLET_COLUMNS), PALLET_COLUMNS),
        start,
        Vec2 { x: 1.0, y: -1.0 },
        TEXTURE_SCALE,
    );

    for ((name, asset), (_, transform)) in assets.iter().zip(transforms) {
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

impl ToAndFromJsonValue for String {
    fn to_json(&self) -> json::JsonValue {
        json::from(self.to_owned())
    }

    fn from_json(json: &json::JsonValue) -> Option<Self> {
        json.as_str().and_then(|str| Some(str.to_owned()))
    }
}

fn grid_save_load_handler(keys: Res<input::ButtonInput<KeyCode>>, mut main_grid: ResMut<MainGrid>) {
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

        main_grid.grid = TileGrid::from_json(&json::parse(&buffer).unwrap()).expect("Grid loaded");
    }
}

fn grid_change_default_handle(
    keys: Res<input::ButtonInput<KeyCode>>,
    mut icon_server: ResMut<MyIconServer>,
) {
    if keys.just_pressed(KeyCode::KeyX) {
        icon_server.default_icon = icon_server
            .next_icon_name_in_cycle(&icon_server.default_icon)
            .to_owned();
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        icon_server.default_icon = icon_server
            .prev_icon_name_in_cycle(&icon_server.default_icon)
            .to_owned();
    }
}

fn grid_change_size(keys: Res<input::ButtonInput<KeyCode>>, mut main_grid: ResMut<MainGrid>) {
    let (cur_rows, cur_cols) = main_grid.grid.size();

    if keys.just_pressed(KeyCode::KeyW) {
        main_grid.grid.resize(cur_rows + 1, cur_cols);
    }

    if keys.just_pressed(KeyCode::KeyS) {
        if cur_rows > 1 {
            main_grid.grid.resize(cur_rows - 1, cur_cols);
        }
    }

    if keys.just_pressed(KeyCode::KeyD) {
        main_grid.grid.resize(cur_rows, cur_cols + 1);
    }

    if keys.just_pressed(KeyCode::KeyA) {
        if cur_cols > 1 {
            main_grid.grid.resize(cur_rows, cur_cols - 1);
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
pub struct ClickedOnMarker; // TODO: Left Right? Middle?

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
    mut query: Query<(&TileMarker, Entity), With<ClickedOnMarker>>,
    mut commands: Commands,
    icon_server: Res<MyIconServer>,
    mut main_grid: ResMut<MainGrid>,
) {
    for (tile_marker, entity) in &mut query {
        info!("Clicked on tile: {:?}", tile_marker.pos);

        main_grid.grid.set(
            tile_marker.pos,
            Some(icon_server.get_selected_name().to_owned()),
        );

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
