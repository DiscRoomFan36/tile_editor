mod icon_server;
mod mouse_stuff;
mod tile_grid;

use std::{
    fs::File,
    io::{Read, Write},
};

use bevy::{input, prelude::*};

use icon_server::*;
use mouse_stuff::*;
use tile_grid::*;

// TODO? Some more window management in the future
const WINDOW_SIZE: (f32, f32) = (1280.0, 720.0);

// TODO: be able to change these at run time.
const SQUARE_SIZE: f32 = 64.0;
const SQUARE_SPACING: f32 = 2.0;

// TODO: Load these from a file and do some hot reloading.
const BASE_TILE_COLOR: Color = Color::hsl(193.0, 0.46, 0.83);
const DEFAULT_TILE_COLOR: Color = Color::hsl(0.0, 0.53, 0.68);

const BASE_PALLET_COLOR: Color = Color::hsl(150.0, 0.7, 0.9);
const HIGHLIGHT_PALLET_COLOR: Color = Color::hsl(50.0, 1.0, 0.55);
const DEFAULT_PALLET_COLOR: Color = Color::hsl(0.0, 0.9, 0.4);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tile Editor".to_string(),
                resolution: WINDOW_SIZE.into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((CameraMousePlugin, IconServerPlugin))
        .insert_resource(MainGrid {
            grid: TileGrid::new(4, 6),
            old_grid: TileGrid::new(0, 0),
        })
        .add_systems(Startup, setup_pallet)
        .add_systems(PreUpdate, (grid_save_load_handler, grid_change_size))
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

// @Think: Put all stuff like this into one big struct with the colors
#[derive(Component)]
struct TileMarker {
    pos: (usize, usize),
}

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
    mut query: Query<(
        &mut Handle<Image>,
        &mut Sprite,
        &ColorHolder,
        &TileMarker,
        Entity,
    )>,
    query_text: Query<Entity, With<TextMarker>>,
    mut commands: Commands,
) {
    if main_grid.grid.size() != main_grid.old_grid.size() {
        for (_, _, _, _, entity) in &mut query {
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
            // TODO: bring this back in some way
            // let rainbow_color = Color::hsl(
            //     360.0 * (x + y * cols) as f32 / (rows * cols) as f32,
            //     0.95,
            //     0.7,
            // );

            let color;
            let texture = if let Some(name) = main_grid.grid.get((x, y)) {
                color = BASE_TILE_COLOR;
                icon_server.get_by_name(name).expect("grid is valid")
            } else {
                color = DEFAULT_TILE_COLOR;
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
                ColorHolder {
                    base_color: BASE_TILE_COLOR,
                    default_color: DEFAULT_TILE_COLOR,
                    ..default()
                },
            ));
        }

        // @Think: be smarter? can you be smarter here? would it even be faster?
        main_grid.old_grid = main_grid.grid.clone();

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

    for (mut handle, mut sprite, color_holder, tile, _entity) in &mut query {
        let pos = tile.pos;

        let old_out_of_date = main_grid.old_grid.get(pos) != main_grid.grid.get(pos);
        let default_changed =
            main_grid.grid.get(pos).is_none() && (*handle != icon_server.get_default_handle());

        // compiler should be smart here, hopefully
        if old_out_of_date || default_changed {
            // info!("Updating sprite!");

            let color;
            let texture = if let Some(name) = main_grid.grid.get(pos) {
                color = color_holder.base_color;
                icon_server.get_by_name(name).expect("grid is valid")
            } else {
                color = color_holder.default_color;
                icon_server.get_default_handle()
            };

            *handle = texture;
            sprite.color = color;

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
        commands.spawn((
            SpriteBundle {
                texture: asset.clone(),
                transform,
                sprite: Sprite {
                    color: BASE_PALLET_COLOR,
                    ..default()
                },
                ..default()
            },
            PalletMarker { name: name.clone() },
            MouseCollider(transform),
            ColorHolder {
                base_color: BASE_PALLET_COLOR,
                highlight_color: HIGHLIGHT_PALLET_COLOR,
                default_color: DEFAULT_PALLET_COLOR,
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

#[derive(Component, Default)]
struct ColorHolder {
    base_color: Color,
    highlight_color: Color,
    default_color: Color,
}

fn update_pallet_highlights(
    mut query: Query<(&PalletMarker, &ColorHolder, &mut Sprite)>,
    icon_server: Res<MyIconServer>,
) {
    for (PalletMarker { name }, color_holder, mut sprite) in &mut query {
        let mut pallet_color = if icon_server.get_selected_name() == name {
            color_holder.highlight_color
        } else {
            color_holder.base_color
        };

        if icon_server.get_default_name() == name {
            // good enough?
            pallet_color = Color::rgb_from_array(
                (pallet_color.rgb_to_vec3() + DEFAULT_PALLET_COLOR.rgb_to_vec3()) / 2.0,
            );
        }

        sprite.color = pallet_color;
    }
}

#[derive(Component)]
pub struct ClickedOnMarker(MouseButton); // TODO: Left Right? Middle?

fn mark_box_clicked(
    mut ev_mouse_collision: EventReader<MouseCollisionEvent>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    for MouseCollisionEvent(entity) in ev_mouse_collision.read() {
        if buttons.just_pressed(MouseButton::Left) {
            commands
                .entity(*entity)
                .insert(ClickedOnMarker(MouseButton::Left));
        }
        if buttons.just_pressed(MouseButton::Right) {
            commands
                .entity(*entity)
                .insert(ClickedOnMarker(MouseButton::Right));
        }
    }
}

fn update_clicked_on_tile(
    mut query: Query<(&TileMarker, &ClickedOnMarker, Entity)>,
    mut commands: Commands,
    icon_server: Res<MyIconServer>,
    mut main_grid: ResMut<MainGrid>,
) {
    for (TileMarker { pos }, ClickedOnMarker(button), entity) in &mut query {
        info!("Clicked on tile: {:?}, {:?}", pos, button);

        match button {
            MouseButton::Left => {
                main_grid
                    .grid
                    .set(*pos, Some(icon_server.get_selected_name().to_owned()));
            }
            MouseButton::Right => {
                main_grid.grid.set(*pos, None);
            }
            _ => panic!("Do not expect other buttons"),
        }

        commands.entity(entity).remove::<ClickedOnMarker>();
    }
}

fn update_clicked_on_pallet(
    mut query: Query<(&PalletMarker, &ClickedOnMarker, Entity)>,
    mut commands: Commands,
    mut icon_server: ResMut<MyIconServer>,
) {
    for (PalletMarker { name }, ClickedOnMarker(button), entity) in &mut query {
        info!("Setting pallet to {name}, {button:?}");

        match button {
            MouseButton::Left => {
                icon_server.set_selected_by_name(name);
            }
            MouseButton::Right => {
                icon_server.set_default_by_name(name);
            }
            _ => panic!("Do not expect other buttons"),
        }

        commands.entity(entity).remove::<ClickedOnMarker>();
    }
}
