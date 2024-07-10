mod mouse_context;
mod tile_grid;
mod icon_server;
mod panel_ui;
mod file_dialog;

use mouse_context::*;
use tile_grid::*;
use icon_server::*;
use panel_ui::*;
use file_dialog::*;

use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use raylib::prelude::*;

const WINDOW_WIDTH  : i32 = 800;
const WINDOW_HEIGHT : i32 = 600;

const SQUARE_SIZE       : i32 = 64;
const SQUARE_SIZE_F     : f32 = SQUARE_SIZE as f32;
const SQUARE_SPACING    : i32 = 10;

const QUICK_SAVE_FILE : &str = "quick-save.json";

// TODO: Remove hardcode? is it good to have something in the pallet at startup?
const PATH            : &str = "./assets/icons";

const TEXT_SIZE    : i32 = 20;
const TEXT_PADDING : i32 = 10;


const BACKGROUND_COLOR                      : Color = Color::LIGHTGRAY;


const PALLET_SELECTED_COLOR : Color = Color::RED;
const PALLET_DEFAULT_COLOR  : Color = Color::BLUE;

const HIGHLIGHT_COLOR                       : Color = Color::ORANGE;

const GRID_START_POSITION   : Vector2 = Vector2::new(100.0, 100.0);
const PALLET_START_POSITION : Vector2 = Vector2::new(10.0, 10.0);

pub struct ImageContainer {
    pub image: Image,
    pub texture: Option<Texture2D>,
}

// these thing have to go together, so why not make it official?
struct GridHandler {
    icon_server: MyIconServer<ImageContainer>,
    grid: TileGrid<String>,
}


fn main() {
    let assets = get_images_from_path(Path::new(PATH));

    let mut grid_handler = GridHandler {
        icon_server: MyIconServer::new(assets),
        grid: TileGrid::new(4, 6),
    };

    // let mut icon_server = MyIconServer::new(assets);

    // let mut grid = TileGrid::new(4, 6);

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tile Editor")
        .build();
    
    rl.set_target_fps(60);
    
    let mut file_dialog_context = FileDialogContext::new();


    // let dirty = true; // TODO: refactor for this
    let mut textures_dirty = true;


    /* -------------------- EVENT LOOP -------------------- */
    while !rl.window_should_close() {

        /* -------------------- KEY EVENT HANDLERS -------------------- */
    
        { // Quick (Save / Load) Handler
            if rl.is_key_pressed(KeyboardKey::KEY_P) {
                println!("Saving Grid!"); // TODO: draw something to the screen

                let json_string = grid_handler.grid.to_json().to_string();
                let mut output = fs::File::create(QUICK_SAVE_FILE).expect("File was created");
                write!(output, "{}", json_string).expect("Write to file");
            }
            if rl.is_key_pressed(KeyboardKey::KEY_L) {
                println!("Loading Saved Grid!"); // TODO: draw something to the screen

                if let Ok(mut input) = fs::File::open(QUICK_SAVE_FILE) {
                    let mut buffer = String::new();
                    input.read_to_string(&mut buffer).expect("Read to buffer");
        
                    grid_handler.grid = TileGrid::from_json(&json::parse(&buffer).unwrap()).expect("Grid loaded");
                } else {
                    println!("No quick save file");
                };
            }
        }

        { // Grid Resizing
            let grid = &mut grid_handler.grid;
            if rl.is_key_pressed(KeyboardKey::KEY_S) {                    grid.resize(grid.rows + 1, grid.cols    )  }
            if rl.is_key_pressed(KeyboardKey::KEY_W) { if grid.rows > 1 { grid.resize(grid.rows - 1, grid.cols    ) }}
            if rl.is_key_pressed(KeyboardKey::KEY_D) {                    grid.resize(grid.rows    , grid.cols + 1)  }
            if rl.is_key_pressed(KeyboardKey::KEY_A) { if grid.cols > 1 { grid.resize(grid.rows    , grid.cols - 1) }}
        }

        { // Selection cycling
            let icon_server = &mut grid_handler.icon_server;
            if rl.is_key_pressed(KeyboardKey::KEY_E) { icon_server.cycle_selected( 1) }
            if rl.is_key_pressed(KeyboardKey::KEY_Q) { icon_server.cycle_selected(-1) }
            if rl.is_key_pressed(KeyboardKey::KEY_X) { icon_server.cycle_default ( 1) }
            if rl.is_key_pressed(KeyboardKey::KEY_Z) { icon_server.cycle_default (-1) }
        }

        { // File dialog
            // TODO: consolidate with mouse events
            if rl.is_key_pressed(KeyboardKey::KEY_O) {
                file_dialog_context.is_open = !file_dialog_context.is_open;
                file_dialog_context.drag_context.is_dragging = false;

                // if there are any "..", reset back to start.
                if file_dialog_context.current_path.components().any(|p| p == Component::ParentDir) {
                    file_dialog_context.current_path = ".".into();
                }

                if file_dialog_context.drag_context.position.x > WINDOW_WIDTH  as f32 || file_dialog_context.drag_context.position.x < 0.0
                || file_dialog_context.drag_context.position.y > WINDOW_HEIGHT as f32 || file_dialog_context.drag_context.position.y < 0.0 {
                    // TODO: do something about this
                    file_dialog_context.drag_context.position = FILE_DIALOG_START_POSITION;
                }
            }
        }

        /* -------------------- MOUSE EVENT HANDLERS -------------------- */
        let mouse_context = MouseContext ::make_context(&rl);
        
        // don't need to check if open, dose that automatically 
        let new_image = file_dialog_context.update(&mouse_context, &mut rl);
        textures_dirty |= grid_handler.add_images(new_image);

        /* -------------------- LOAD TEXTURES -------------------- */
        if textures_dirty {
            for (_, image_container) in grid_handler.icon_server.assets.iter_mut() {
                let mut image = image_container.image.clone();
                image.resize(SQUARE_SIZE_F as i32, SQUARE_SIZE_F as i32);
                
                let texture = rl.load_texture_from_image(&thread, &image).expect("load texture");
                image_container.texture = Some(texture)
            }

            textures_dirty = false;
        }
        /* -------------------- LOAD TEXTURES -------------------- */

        grid_handler.update(&mouse_context);
        
        /* -------------------- DRAWING -------------------- */
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(BACKGROUND_COLOR);


        // TODO: Move the grid out of the way
        /* -------------------- DRAW GRID -------------------- */
        let grid_panel = grid_handler.to_grid_panel();
        grid_panel.draw_panel(&mut d, &mouse_context);
        /* -------------------- DRAW GRID -------------------- */

        /* -------------------- DRAW PALLET -------------------- */
        let pallet_panel = grid_handler.to_pallet_panel();
        pallet_panel.draw_panel(&mut d, &mouse_context);
        /* -------------------- DRAW PALLET -------------------- */



        /* -------------------- FILE DIALOG -------------------- */
        file_dialog_context
            .to_panel(&mut d)
            .draw_panel(&mut d, &mouse_context);
        /* -------------------- FILE DIALOG -------------------- */
    }
}

impl ToAndFromJsonValue for String {
    fn to_json(&self) -> json::JsonValue { json::from(self.to_owned()) }
    fn from_json(json: &json::JsonValue) -> Option<Self> {
        json.as_str().and_then(|str| Some(str.to_owned()))
    }
}

fn get_image_from_path(path: &Path) -> (String, ImageContainer) {
    let name = path.to_str().expect("Valid path").to_string();

    let image = raylib::texture::Image::load_image(&name).expect("Is valid image");

    (name, ImageContainer { image, texture: None })
}
fn get_images_from_path(path: &Path) -> Vec<(String, ImageContainer)> {
    let paths = fs::read_dir(path).expect("Valid directory");

    let names: Vec<_> = paths
        .map(|path| path.unwrap())
        .map(|path|
            path.path().to_str().expect("Valid path").to_string()
        )
        .collect();
    
    let images: Vec<_> = names
        .iter()
        .map(|path|
            raylib::texture::Image::load_image(&path).expect("Is Valid Image")
        )
        .collect();

    names.into_iter()
        .zip(images)
        .map(|(s, image)|
            (s, ImageContainer { image, texture: None }
        ))
        .collect()
}

impl<'a> GridHandler {
    fn to_grid_panel(&'a self) -> GridPanel<&'a Texture2D> {
        let (rows, cols) = self.grid.size();
        
        let mut panel = GridPanel::new_custom(
            GRID_START_POSITION,
            true, cols,
            64, 64, 10,
            Some(Color::ORANGE), None
        );

        for i in 0..rows*cols {
            let image_container = if let Some(name) = self.grid.get_from_index(i) {
                self.icon_server.get_by_name(name).expect("Name exist in icon server")
            } else {
                self.icon_server.get_default_handle()
            };
    
            let Some(texture) = image_container.texture.as_ref() else {
                panel.add_none();
                continue;
            };
    
            panel.add(texture);
        }
    
        return panel;
    }

    fn to_pallet_panel(&self) -> GridPanel<&Texture2D> {
        let mut panel = GridPanel::new_custom(
            PALLET_START_POSITION,
            true, 3,
            SQUARE_SIZE, SQUARE_SIZE, SQUARE_SPACING,
            Some(HIGHLIGHT_COLOR), None
        );

        for (name, image_container) in self.icon_server.assets.iter() {
            let Some(texture) = image_container.texture.as_ref() else {
                panel.add_none();
                continue;
            };

            let mut highlights = vec![];

            if self.icon_server.get_default_name()  == name { highlights.push(PALLET_DEFAULT_COLOR) }
            if self.icon_server.get_selected_name() == name { highlights.push(PALLET_SELECTED_COLOR) }

            panel.add_with_highlight(texture, &highlights);
        }

        return panel;
    }

    fn update(&mut self, mouse_context: &MouseContext) {
        self.update_pallet(mouse_context);
        self.update_grid(mouse_context);
    }

    fn update_grid(&mut self, mouse_context: &MouseContext) {
        let grid_pallet = self.to_grid_panel();

        let id = grid_pallet.get_hovered_id(mouse_context);

        let Some(id) = id else { return; };

        let pos = index_to_pos(id, self.grid.size());

        if mouse_context.mouse_left_pressed {
            self.grid.set(pos, Some(self.icon_server.get_selected_name().to_string()));
        }
        if mouse_context.mouse_right_pressed {
            self.grid.set(pos, None);
        }
    }

    pub fn update_pallet(&mut self, mouse_context: &MouseContext) {
        let pallet_panel = self.to_pallet_panel();

        let Some(id) = pallet_panel.get_hovered_id(&mouse_context) else { return; };

        let name = self.icon_server.assets[id].0.clone();

        if mouse_context.mouse_left_pressed {
            self.icon_server.set_selected_by_name(&name);
        }
        if mouse_context.mouse_right_pressed {
            self.icon_server.set_default_by_name(&name);
        }
    }

    // returns textures_dirty
    fn add_images(&mut self, new_image: Option<PathBuf>) -> bool {
        let Some(path) = new_image else { return false; };
            // TODO: this should be simpler
        if path.is_dir() {
            self.icon_server.load_icons(&mut get_images_from_path(&path));
        } else {
            self.icon_server.load_icon(get_image_from_path(&path));
        }
        return true; // textures_dirty = true; // Remember to call when adding images
    }
}
