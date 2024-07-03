mod tile_grid;
mod icon_server;
mod panel_ui;

use tile_grid::*;
use icon_server::*;
use panel_ui::*;

use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use raylib::prelude::*;
use raylib::consts::{KeyboardKey, MouseButton};

const WINDOW_WIDTH  : i32 = 800;
const WINDOW_HEIGHT : i32 = 600;

const SQUARE_SIZE       : f32 = 64.0;
const SQUARE_SPACING    : f32 = 10.0;
const HIGHLIGHT_PADDING : f32 = SQUARE_SPACING / 2.0;

const QUICK_SAVE_FILE : &str = "quick-save.json";

// TODO: Remove hardcode? is it good to have something in the pallet at startup?
const PATH            : &str = "./assets/icons";

const PALLET_PER_ROW : usize = 3;

const TEXT_SIZE    : i32 = 30;
const TEXT_PADDING : i32 = 10;

const FILE_DIALOG_SELECT_FOLDER_TEXT : &str = "Select Folder";

const BACKGROUND_COLOR                      : Color = Color::LIGHTGRAY;

const GRID_TEXTURE_TINT                     : Color = Color::WHITE;

const HIGHLIGHT_COLOR                       : Color = Color::ORANGE;
const PALLET_SELECTED_COLOR                 : Color = Color::RED;
const PALLET_DEFAULT_COLOR                  : Color = Color::BLUE;
const PALLET_TEXTURE_TINT                   : Color = GRID_TEXTURE_TINT;

const FILE_DIALOG_CURRENT_FOLDER_COLOR      : Color = Color::MAROON;
const FILE_DIALOG_CURRENT_FOLDER_TEXT_COLOR : Color = Color::GOLDENROD;
const FILE_DIALOG_BACKING_BOX_COLOR         : Color = Color::DARKGRAY;

const FILE_DIALOG_LABEL_HOVER_COLOR         : Color = Color::ORANGE;
const FILE_DIALOG_LABEL_TEXT_COLOR          : Color = Color::GOLD;

const FILE_DIALOG_SELECT_BACKING_COLOR      : Color = Color::GREEN;
const FILE_DIALOG_SELECT_HOVER_COLOR        : Color = Color::WHEAT;
const FILE_DIALOG_SELECT_TEXT_COLOR         : Color = Color::BLACK;

const FILE_DIALOG_START_POSITION : Vector2 = Vector2 { x: 100.0, y: 100.0 };

struct ImageContainer {
    image: Image,
    texture: Option<Texture2D>,
}

#[derive(Debug)]
struct MouseContext {
    mouse_pos   :  Vector2,
    mouse_delta : Vector2,
    mouse_left_pressed  : bool,
    mouse_left_released : bool,
    mouse_right_pressed : bool,

    hovering_over_pallet             : Vec<bool>,
    hovering_over_grid               : Vec<bool>,

    over_file_dialog : bool,
}

struct FileDialogContext {
    is_open: bool,
    current_path: PathBuf,
    // width: i32,
    // menu_position: Vector2,
    // is_dragging: bool,
    // menu_started_dragging_position: Vector2,

    drag_context: PanelUiDragContext,
}

fn main() {
    let assets = get_images_from_path(Path::new(PATH));

    let mut icon_server = MyIconServer::new(assets);

    let mut grid = TileGrid::new(4, 6);

    // TODO: be smarter with this
    let start_pos = Vector2::new(100.0, 100.0);

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Tile Editor")
        .build();
    
    rl.set_target_fps(60);
    
    let mut file_dialog_context = FileDialogContext {
        is_open: false,
        current_path: ".".into(),
        // width: rl.measure_text(FILE_DIALOG_SELECT_FOLDER_TEXT, TEXT_SIZE) + TEXT_PADDING * 2,
        // menu_position: FILE_DIALOG_START_POSITION,
        // is_dragging: false,
        // menu_started_dragging_position: Vector2::zero(),

        drag_context: PanelUiDragContext::new(FILE_DIALOG_START_POSITION),
    };


    // let dirty = true; // TODO: refactor for this
    let mut textures_dirty = true;

    // TESTING
    // TESTING
    // TESTING
    // TESTING
    // let mut a_drag_context = PanelUiDragContext::new(Vector2::new(100.0, 100.0));

    // let mut panel_drag = PanelUiDragContext::new(Vector2::new(300.0, 100.0));
    // TESTING
    // TESTING
    // TESTING
    // TESTING

    /* -------------------- EVENT LOOP -------------------- */
    while !rl.window_should_close() {

        /* -------------------- KEY EVENT HANDLERS -------------------- */
    
        { // Quick (Save / Load) Handler
            if rl.is_key_pressed(KeyboardKey::KEY_P) {
                println!("Saving Grid!"); // TODO: draw something to the screen

                let json_string = grid.to_json().to_string();
                let mut output = fs::File::create(QUICK_SAVE_FILE).expect("File was created");
                write!(output, "{}", json_string).expect("Write to file");
            }
            if rl.is_key_pressed(KeyboardKey::KEY_L) {
                println!("Loading Saved Grid!"); // TODO: draw something to the screen

                if let Ok(mut input) = fs::File::open(QUICK_SAVE_FILE) {
                    let mut buffer = String::new();
                    input.read_to_string(&mut buffer).expect("Read to buffer");
        
                    grid = TileGrid::from_json(&json::parse(&buffer).unwrap()).expect("Grid loaded");
                } else {
                    println!("No quick save file");
                };
            }
        }

        { // Grid Resizing
            if rl.is_key_pressed(KeyboardKey::KEY_S) {                    grid.resize(grid.rows + 1, grid.cols    )  }
            if rl.is_key_pressed(KeyboardKey::KEY_W) { if grid.rows > 1 { grid.resize(grid.rows - 1, grid.cols    ) }}
            if rl.is_key_pressed(KeyboardKey::KEY_D) {                    grid.resize(grid.rows    , grid.cols + 1)  }
            if rl.is_key_pressed(KeyboardKey::KEY_A) { if grid.cols > 1 { grid.resize(grid.rows    , grid.cols - 1) }}
        }

        { // Selection cycling
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
                    file_dialog_context.drag_context.position = FILE_DIALOG_START_POSITION;
                }
            }
        }

        /* -------------------- MOUSE EVENT HANDLERS -------------------- */
        let mut mouse_context = MouseContext {
            mouse_pos           : rl.get_mouse_position(),
            mouse_delta         : rl.get_mouse_delta(),
            mouse_left_pressed  : rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            mouse_left_released : rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
            mouse_right_pressed : rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT),
    
            hovering_over_pallet             : vec![false; icon_server.assets.len()],
            hovering_over_grid               : vec![false; grid.rows*grid.cols],

            over_file_dialog : false,
        };
        
        // Must be called first
        // update_file_dialog_mouse_events(&rl, &mut file_dialog_context, &mut mouse_context, &mut icon_server, &mut textures_dirty);

        update_pallet_mouse_events(&mut mouse_context, &mut icon_server);

        update_grid_mouse_events(&mut mouse_context, &mut icon_server, &mut grid, start_pos);

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

        d.clear_background(BACKGROUND_COLOR);

        // TODO: Move the grid out of the way
        /* -------------------- DRAW GRID -------------------- */
        for i in 0..grid.rows*grid.cols {
            let (x, y) = index_to_pos(i, grid.size());

            let rec = new_square(start_pos, (x, y));
            
            /* -------------------- ON HOVER GRID -------------------- */
            if mouse_context.hovering_over_grid[i] {
                // draw some highlighting around the hovered rectangle
                d.draw_rectangle_rec(pad_rectangle(rec, HIGHLIGHT_PADDING), HIGHLIGHT_COLOR);
            }
    
            let image_container = if let Some(name) = grid.get((x, y)) {
                icon_server.get_by_name(name).expect("Name exist in icon server")
            } else {
                icon_server.get_default_handle()
            };

            d.draw_texture(
                image_container.texture.as_ref().unwrap(),
                rec.x as i32, rec.y as i32, GRID_TEXTURE_TINT
            );
        }
    
        /* -------------------- DRAW PALLET -------------------- */
        for i in 0..icon_server.assets.len() {
            let name = icon_server.assets[i].0.clone();
            let (x, y) = index_to_pos(i, (999, PALLET_PER_ROW));

            let rec = new_square(Vector2::new(10.0, 10.0), (x, y));
            let padding_rec = pad_rectangle(rec, HIGHLIGHT_PADDING);

            // Default and selected highlighting
            let is_default = icon_server.get_default_name() == name;
            let is_selected = icon_server.get_selected_name() == name;


            if is_default {
                d.draw_rectangle_rec(padding_rec, PALLET_DEFAULT_COLOR);
            }    
            if is_selected {
                let mut padding_rec = padding_rec;
                if is_default { padding_rec.width /= 2.0; }
                d.draw_rectangle_rec(padding_rec, PALLET_SELECTED_COLOR);
            }

            if *mouse_context.hovering_over_pallet.get(i).unwrap_or(&false) {
                if is_default || is_selected {
                    let top_stripe = Rectangle {
                        x: padding_rec.x + padding_rec.width / 3.0,  y: padding_rec.y,
                        width: padding_rec.width / 3.0,              height: padding_rec.height,
                    };
                    let middle_stripe = Rectangle {
                        x: padding_rec.x,                            y: padding_rec.y + padding_rec.height / 3.0,
                        width: padding_rec.width,                    height: padding_rec.height / 3.0,
                    };
                    d.draw_rectangle_rec(top_stripe, HIGHLIGHT_COLOR);
                    d.draw_rectangle_rec(middle_stripe, HIGHLIGHT_COLOR);
                } else {
                    d.draw_rectangle_rec(pad_rectangle(rec, HIGHLIGHT_PADDING), HIGHLIGHT_COLOR);
                }
            }

            let (_, image_container) = &icon_server.assets[i];

            d.draw_texture(
                image_container.texture.as_ref().unwrap(),
                rec.x as i32, rec.y as i32, PALLET_TEXTURE_TINT
            );
        }

        /* -------------------- FILE DIALOG -------------------- */
    
        // Testing new ui thing here
        // Testing new ui thing here
        // Testing new ui thing here
        

        if file_dialog_context.is_open {
            let mut file_dialog_panel = PanelColumn::new_draggable(file_dialog_context.drag_context);

            let mut header = TextPanel::new();
            header.add_text_button_by_d(file_dialog_context.current_path.to_str().unwrap(), &mut d);
            file_dialog_panel.add_panel(header, true);

            let mut file_list = TextPanel::new();
            let file_names = list_directory(&file_dialog_context.current_path);
            // Hack.
            let v: Vec<&str> = file_names.iter().map(|x| x.as_ref()).collect();
            file_list.add_text_buttons_by_d(&v, &mut d);
            file_dialog_panel.add_panel(file_list, false);

            let mut select_folder_button = TextPanel::new();
            select_folder_button.add_text_button_by_d(FILE_DIALOG_SELECT_FOLDER_TEXT, &mut d);
            file_dialog_panel.add_panel(select_folder_button, false);

            file_dialog_context.drag_context = file_dialog_panel.do_dragging(&mouse_context);

            if mouse_context.mouse_left_pressed {
                let hovered = file_dialog_panel.get_hovered_id_recursively(&mouse_context);

                // handle switch folders,
                if hovered.get(0) == Some(&1) {
                    if let Some(i) = hovered.get(1) {
                        let file = &file_names[*i];
                        let path = file_dialog_context.current_path.join(&file);
            
                        assert!(path.exists());
                        if path.is_dir() {
                            let new_path = file_dialog_context.current_path.join(&file);
                            file_dialog_context.current_path = clean_path(&new_path);
                        } else {
                            icon_server.load_icon(get_image_from_path(&path));
                            textures_dirty = true;
                            
                            file_dialog_context.is_open = false;
                        }
                    }
                }

                // handle select thing,
                if hovered.get(0) == Some(&2) && hovered.len() == 2 {
                    icon_server.load_icons(&mut get_images_from_path(&file_dialog_context.current_path));
                    textures_dirty = true; // Remember to call when adding images
        
                    file_dialog_context.is_open = false; // Close it because we done here
                }
            }


            // TODO: reload
            file_dialog_panel.draw_panel(&mut d, &mouse_context)
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

fn pad_rectangle_ex(rec: Rectangle, left: f32, right: f32, top: f32, bottom: f32) -> Rectangle {
    Rectangle {
        x:      rec.x      - left,
        y:      rec.y      - top,
        width:  rec.width  + right * 2.0,
        height: rec.height + bottom * 2.0,
    }
}
fn pad_rectangle(rec: Rectangle, padding: f32) -> Rectangle {
    pad_rectangle_ex(rec, padding, padding, padding, padding)
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

fn list_directory(path: &Path) -> Vec<String> {
    let dir = fs::read_dir(path).expect("Path is valid");
    let mut file_names: Vec<_> = dir
        .map(|path| {
            path.unwrap()
                .file_name()
                .into_string()
                .unwrap()
        })
        .filter(|name| !name.starts_with(".")) // filter out hidden files
        .filter(|name| {
            if path.join(name).is_dir() { return true; }
            // filter out things that aren't drawable (aka png's)
            name.split_once(".")
                .map(|(_, end)| end == "png")
                .unwrap_or_default()
        })
        .collect();

    file_names.reverse();

    file_names.insert(0, "..".into());

    // TODO: sort by type then name

    file_names
}

fn clean_path(path: &Path) -> PathBuf {
    // doesn't (or can't) handle the case where you leave the "." folder then enter back into it.
    // Also no symlinks

    let mut sections: Vec<_> = path.to_str().unwrap().split("/").collect();
    
    // remove "dir/../"
    let mut i = 1;
    while i < sections.len() {
        if sections[i] == ".." && sections[i - 1] != "." {
            sections.remove(i);
            sections.remove(i - 1);
            i -= 1;
        } else {
            i += 1
        }
    }

    // reconstruct
    return PathBuf::from(sections.join("/"));
}

fn update_pallet_mouse_events(
    mouse_context: &mut MouseContext,
    icon_server: &mut MyIconServer<ImageContainer>,
) {
    if mouse_context.over_file_dialog { return; }

    for i in 0..icon_server.assets.len() {
        let name = icon_server.assets[i].0.clone();
        let (x, y) = index_to_pos(i, (999, PALLET_PER_ROW));

        let rec = new_square(Vector2::new(10.0, 10.0), (x, y));

        if rec.check_collision_point_rec(mouse_context.mouse_pos) {
            mouse_context.hovering_over_pallet[i] = true;
            if mouse_context.mouse_left_pressed {
                icon_server.set_selected_by_name(&name);
            }
            if mouse_context.mouse_right_pressed {
                icon_server.set_default_by_name(&name);
            }
        }
    }
}

fn update_grid_mouse_events(
    mouse_context: &mut MouseContext,
    icon_server: &mut MyIconServer<ImageContainer>,
    grid: &mut TileGrid<String>,
    start_pos: Vector2, // TODO: Refactor into context
) {
    if mouse_context.over_file_dialog { return; }
    
    for i in 0..grid.rows*grid.cols {
        let (x, y) = index_to_pos(i, grid.size());
        let rec = new_square(start_pos, (x, y));
        
        if rec.check_collision_point_rec(mouse_context.mouse_pos) {
            mouse_context.hovering_over_grid[i] = true;
            if mouse_context.mouse_left_pressed {
                grid.set((x, y), Some(icon_server.get_selected_name().to_string()));
            }
            if mouse_context.mouse_right_pressed {
                grid.set((x, y), None);
            }
        }
    }
}
