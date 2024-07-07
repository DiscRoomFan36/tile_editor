use std::fs;
use std::path::{Path, PathBuf};

use raylib::prelude::*;

use crate::{MouseContext, TEXT_PADDING, TEXT_SIZE};

use crate::panel_ui::*;

const ITEM_PADDING : i32 = 4;

const FILE_DIALOG_SELECT_FOLDER_TEXT : &str = "Select Folder";

const FILE_DIALOG_CURRENT_FOLDER_COLOR      : Color = Color::MAROON;
const FILE_DIALOG_CURRENT_FOLDER_TEXT_COLOR : Color = Color::GOLDENROD;

const FILE_DIALOG_LABEL_BACKGROUND_COLOR    : Color = Color::DARKGRAY;

const FILE_DIALOG_LABEL_HOVER_COLOR         : Color = Color::ORANGE;
const FILE_DIALOG_LABEL_TEXT_COLOR          : Color = Color::GOLD;

const FILE_DIALOG_SELECT_BACKGROUND_COLOR   : Color = Color::GREEN;
const FILE_DIALOG_SELECT_HOVER_COLOR        : Color = Color::WHEAT;
const FILE_DIALOG_SELECT_TEXT_COLOR         : Color = Color::BLACK;

pub const FILE_DIALOG_START_POSITION : Vector2 = Vector2 { x: 100.0, y: 100.0 };


pub struct FileDialogContext {
    pub is_open: bool,
    pub current_path: PathBuf,

    pub drag_context: PanelUiDragContext,
}

impl FileDialogContext {
	pub fn new() -> Self {
		FileDialogContext {
			is_open: false,
			current_path: ".".into(),
	
			drag_context: PanelUiDragContext::new(FILE_DIALOG_START_POSITION),
		}
	}

	pub fn to_panel(&self, rl: &mut impl CanMeasureText) -> PanelColumn<TextPanel> {
		let mut file_dialog_panel = PanelColumn::new_draggable(self.drag_context);
		
		if self.is_open == false { return file_dialog_panel; }

		{ // Current folder text
			let mut header = TextPanel::new_custom(
				TEXT_SIZE,
				TEXT_PADDING,
				ITEM_PADDING,
				FILE_DIALOG_CURRENT_FOLDER_COLOR,
				FILE_DIALOG_CURRENT_FOLDER_TEXT_COLOR,
				None
			);
			header.add_text_button(self.current_path.to_str().unwrap(), rl);
			file_dialog_panel.add_panel(header, true);
		}

		{ // File labels
			let mut file_list = TextPanel::new_custom(
				TEXT_SIZE,
				TEXT_PADDING,
				ITEM_PADDING,
				FILE_DIALOG_LABEL_BACKGROUND_COLOR,
				FILE_DIALOG_LABEL_TEXT_COLOR,
				Some(FILE_DIALOG_LABEL_HOVER_COLOR)
			);
			let file_names = list_directory(&self.current_path);
			// Hack.
			let v: Vec<&str> = file_names.iter().map(<_>::as_ref).collect();
			file_list.add_text_buttons(&v, rl);
			file_dialog_panel.add_panel(file_list, false);
		}

		{ // Select Folder Button
			let mut select_folder_button = TextPanel::new_custom(
				TEXT_SIZE,
				TEXT_PADDING,
				ITEM_PADDING,
				FILE_DIALOG_SELECT_BACKGROUND_COLOR,
				FILE_DIALOG_SELECT_TEXT_COLOR,
				Some(FILE_DIALOG_SELECT_HOVER_COLOR)
			);
			select_folder_button.add_text_button(FILE_DIALOG_SELECT_FOLDER_TEXT, rl);
			file_dialog_panel.add_panel(select_folder_button, false);
		}

		return file_dialog_panel;
	}

	// return an image to load
	pub fn update(&mut self, mouse_context: &MouseContext, rl: &mut impl CanMeasureText) -> Option<PathBuf> {
		let mut file_dialog_panel = self.to_panel(rl);

		let file_names = list_directory(&self.current_path);

		self.drag_context = file_dialog_panel.do_dragging(&mouse_context);

		if mouse_context.mouse_left_pressed {
			let hovered = file_dialog_panel.get_hovered_id_recursively(&mouse_context);

			// handle switch folders,
			if hovered.get(0) == Some(&1) {
				if let Some(i) = hovered.get(1) {
					let file = &file_names[*i];
					let path = self.current_path.join(&file);
		
					assert!(path.exists());
					if path.is_dir() {
						let new_path = self.current_path.join(&file);
						self.current_path = clean_path(&new_path);
					} else {
						self.is_open = false;

						return Some(path);
					}
				}
			}

			// handle select thing,
			if hovered.get(0) == Some(&2) && hovered.len() == 2 {
				self.is_open = false; // Close it because we done here

				return Some(self.current_path.clone());
			}
		}

		return None;
	}

}

pub fn list_directory(path: &Path) -> Vec<String> {
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

pub fn clean_path(path: &Path) -> PathBuf {
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