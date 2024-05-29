mod tile_grid;
mod icon_server;

use tile_grid::*;
use icon_server::*;

use std::fs;
use std::io::{Read, Write};

use raylib::prelude::*;
use raylib::consts::{KeyboardKey, MouseButton};

const SQUARE_SIZE: f32 = 64.0;
const SQUARE_SPACING: f32 = 10.0;

const HIGHLIGHT_PADDING: f32 = 5.0;

const HIGHLIGHT_COLOR: Color = Color::ORANGE;

const PALLET_SELECTED_COLOR: Color = Color::RED;
const PALLET_DEFAULT_COLOR: Color = Color::BLUE;

const PATH: &str = "./assets/icons";

struct ImageContainer {
    image: Image,
    texture: Option<Texture2D>,
}

fn main() {
    let assets = get_images_from_path(PATH)
        .drain(..)
        .map(|(s, image)|
            (s, ImageContainer { image, texture: None }
        ))
        .collect::<Vec<_>>();

    let mut icon_server = MyIconServer::new(assets);

    let mut grid: TileGrid<String> = TileGrid::new(4, 6);

    // TODO: be smarter with this
    let start_pos = Vector2::new(100.0, 100.0);

    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Tile Editor")
        .build();
    
    rl.set_target_fps(60);
    
    let mut textures_dirty = true;
    // let dirty = true; // TODO: refactor for this

    /* -------------------- EVENT LOOP -------------------- */
    while !rl.window_should_close() {

        /* -------------------- KEY EVENT HANDLERS -------------------- */
    
        // QUICK (SAVE / LOAD) Handler
        const QUICK_SAVE_FILE: &str = "quick-save.json";
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            println!("Saving Grid!");

            let json_string = grid.to_json().to_string();
            let mut output = fs::File::create(QUICK_SAVE_FILE).expect("File was created");
            write!(output, "{}", json_string).expect("Write to file");
        }
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            println!("Loading Saved Grid!");

            if let Ok(mut input) = fs::File::open(QUICK_SAVE_FILE) {
                let mut buffer = String::new();
                input.read_to_string(&mut buffer).expect("Read to buffer");
    
                grid = TileGrid::from_json(&json::parse(&buffer).unwrap()).expect("Grid loaded");
            } else {
                println!("No quick save file");
            };
        }

        // Grid Resizing
        if rl.is_key_pressed(KeyboardKey::KEY_S) {                    grid.resize(grid.rows + 1, grid.cols    )  }
        if rl.is_key_pressed(KeyboardKey::KEY_W) { if grid.rows > 1 { grid.resize(grid.rows - 1, grid.cols    ) }}
        if rl.is_key_pressed(KeyboardKey::KEY_D) {                    grid.resize(grid.rows    , grid.cols + 1)  }
        if rl.is_key_pressed(KeyboardKey::KEY_A) { if grid.cols > 1 { grid.resize(grid.rows    , grid.cols - 1) }}

        // selection cycling
        if rl.is_key_pressed(KeyboardKey::KEY_E) { icon_server.cycle_selected( 1) }
        if rl.is_key_pressed(KeyboardKey::KEY_Q) { icon_server.cycle_selected(-1) }
        if rl.is_key_pressed(KeyboardKey::KEY_X) { icon_server.cycle_default ( 1) }
        if rl.is_key_pressed(KeyboardKey::KEY_Z) { icon_server.cycle_default (-1) }

        let mouse_pos = rl.get_mouse_position();


        /* -------------------- LOAD TEXTURES -------------------- */
        if textures_dirty {
            for (_, image_container) in icon_server.assets.iter_mut() {
                let mut image = image_container.image.clone();
                image.resize(SQUARE_SIZE as i32, SQUARE_SIZE as i32);
                
                let texture = rl.load_texture_from_image(&thread, &image).expect("load texture");
                image_container.texture = Some(texture)
            }

            textures_dirty = false;
        }
        
        /* -------------------- DRAWING -------------------- */
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::LIGHTGRAY);


        /* -------------------- DRAW PALLET -------------------- */
        const PER_ROW: usize = 3;
        for i in 0..icon_server.assets.len() {
            let name = icon_server.assets[i].0.clone();
            let (x, y) = index_to_pos(i, (999, PER_ROW));

            let rec = new_square(Vector2::new(10.0, 10.0), (x, y));

            /* -------------------- ON HOVER PALLET -------------------- */
            if rec.check_collision_point_rec(mouse_pos) {
                // draw some highlighting around the hovered rectangle
                d.draw_rectangle_rec(
                    pad_rectangle(rec, HIGHLIGHT_PADDING),
                    HIGHLIGHT_COLOR
                );

                if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    icon_server.set_selected_by_name(&name);
                }
                if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                    icon_server.set_default_by_name(&name);
                }
            }

            // Default and selected highlighting
            let is_default = icon_server.get_default_name() == name;
            if is_default {
                d.draw_rectangle_rec(pad_rectangle(rec, HIGHLIGHT_PADDING), PALLET_DEFAULT_COLOR);
            }

            if icon_server.get_selected_name() == name {
                let mut rec = pad_rectangle(rec, HIGHLIGHT_PADDING);
                if is_default { rec.width /= 2.0; }
                d.draw_rectangle_rec(rec, PALLET_SELECTED_COLOR);
            }

            let (_, image_container) = &icon_server.assets[i];

            d.draw_texture(image_container.texture.as_ref().unwrap(), rec.x as i32, rec.y as i32, Color::WHITE);
        }


        /* -------------------- DRAW GRID -------------------- */
        for i in 0..grid.rows*grid.cols {
            let (x, y) = index_to_pos(i, grid.size());

            let rec = new_square(start_pos, (x, y));
            
            /* -------------------- ON HOVER GRID -------------------- */
            if rec.check_collision_point_rec(mouse_pos) {
                // draw some highlighting around the hovered rectangle
                d.draw_rectangle_rec(
                    pad_rectangle(rec, HIGHLIGHT_PADDING),
                    HIGHLIGHT_COLOR
                );
    
                if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    grid.set((x, y), Some(icon_server.get_selected_name().to_string()));
                }
                if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                    grid.set((x, y), None);
                }
            }
    
            let image_container = if let Some(name) = grid.get((x, y)) {
                icon_server.get_by_name(name).expect("Name exist in icon server")
            } else {
                icon_server.get_default_handle()
            };
    
            d.draw_texture(image_container.texture.as_ref().unwrap(), rec.x as i32, rec.y as i32, Color::WHITE);
        }
    }
}

fn new_square(start_pos: Vector2, pos: (usize, usize)) -> Rectangle {
    let (x, y) = pos;
    Rectangle {
        x: start_pos.x + x as f32 * (SQUARE_SIZE + SQUARE_SPACING),
        y: start_pos.y + y as f32 * (SQUARE_SIZE + SQUARE_SPACING),
        width: SQUARE_SIZE, height: SQUARE_SIZE,
    }
}

fn pad_rectangle(rec: Rectangle, padding: f32) -> Rectangle {
    Rectangle {
        x:      rec.x      - padding,
        y:      rec.y      - padding,
        width:  rec.width  + padding * 2.0,
        height: rec.height + padding * 2.0,
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

fn get_images_from_path(path: &str) -> Vec<(String, Image)> {
    let paths = fs::read_dir(path).expect("Valid directory");

    let names = paths
        .map(|path| path.unwrap())
        .map(|path| {
            path.path()
                .file_name()
                .and_then(|p| p.to_str())
                .and_then(|p| Some(p.to_string()))
                .unwrap()
        })
        .map(|file| format!("{PATH}/{file}"));
    
    names
        .map(|name| (
            name.clone(),
            raylib::texture::Image::load_image(&name).expect("Image exists")
        )
        ).collect()
}