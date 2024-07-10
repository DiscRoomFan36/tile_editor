pub struct MyIconServer<T> {
    pub assets: Vec<(String, T)>, // TODO: Remove pub at some point? make hidden ect...
    selected: String,     // TODO: use str
    default_icon: String, // TODO: use str
}

impl<T> MyIconServer<T> {
	pub fn new(assets: Vec<(String, T)>) -> Self {
		assert!(assets.len() > 1);
		Self {
			default_icon: assets[0].0.clone(),
			selected: assets[1].0.clone(),
			assets,
		}
	}

    pub fn load_icons(&mut self, assets: &mut Vec<(String, T)>) {
        self.assets.append(assets);
    }

    pub fn load_icon(&mut self, asset: (String, T)) {
        self.assets.push(asset);
    }

    pub fn get_selected_name(&self) -> &str {
        return &self.selected;
    }

    pub fn get_default_name(&self) -> &str {
        return &self.default_icon;
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.assets
            .iter()
            .find(|(asset_name, _)| name == asset_name)
            .map(|(_, handle)| handle)
    }

    pub fn get_default_handle(&self) -> &T {
        self.get_by_name(&self.default_icon)
            .expect("self.default_icon is valid")
    }

    // TODO
    // pub fn set_selected_by_id(&mut self, id: usize) { todo() }
    pub fn set_selected_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.selected = name.to_owned();
    }

    // TODO
    // pub fn set_default_by_id(&mut self, id: usize) { todo() }
    pub fn set_default_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.default_icon = name.to_owned();
    }

    fn cycle_icon(&self, name: &str, count: i32) -> &str {
        let index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (asset_name, _))| *asset_name == name)
            .map(|(i, _)| i)
            .unwrap();

        let len = self.assets.len() as i32;

        let index = ((index as i32 + count) % len + len) % len;

        self.assets
            .get(index as usize)
            .map(|(name, _)| name)
            .unwrap()
    }

    pub fn cycle_selected(&mut self, count: i32) {
        self.selected = self.cycle_icon(&self.selected, count).to_owned();
    }

    pub fn cycle_default(&mut self, count: i32) {
        self.default_icon = self.cycle_icon(&self.default_icon, count).to_owned();
    }
}


use raylib::prelude::*;

use crate::{GridPanel, MouseContext, PanelLike, HIGHLIGHT_COLOR, SQUARE_SIZE, SQUARE_SPACING};

const PALLET_SELECTED_COLOR                 : Color = Color::RED;
const PALLET_DEFAULT_COLOR                  : Color = Color::BLUE;

const PALLET_START_POSITION : Vector2 = Vector2::new(10.0, 10.0);


pub struct ImageContainer {
    pub image: Image,
    pub texture: Option<Texture2D>,
}

impl MyIconServer<ImageContainer> {
    pub fn to_pallet_panel(&self) -> GridPanel<&Texture2D> {
        let mut panel = GridPanel::new_custom(
            PALLET_START_POSITION,
            true, 3,
            SQUARE_SIZE, SQUARE_SIZE, SQUARE_SPACING,
            Some(HIGHLIGHT_COLOR), None
        );

        for (name, image_container) in self.assets.iter() {
            let texture = image_container.texture.as_ref();

            if texture.is_none() { panel.add_none(); continue; }

            let mut highlights = vec![];

            if self.get_default_name()  == name { highlights.push(PALLET_DEFAULT_COLOR) }
            if self.get_selected_name() == name { highlights.push(PALLET_SELECTED_COLOR) }

            panel.add_with_highlight(texture.unwrap(), &highlights);
        }

        return panel;
    }

    pub fn update_pallet(&mut self, mouse_context: &MouseContext) {
        let pallet_panel = self.to_pallet_panel();

        let id = pallet_panel.get_hovered_id(&mouse_context);

        if id.is_none() { return; }
        let id = id.unwrap();
        
        let name = self.assets[id].0.clone();

        if mouse_context.mouse_left_pressed {
            self.set_selected_by_name(&name);
        }
        if mouse_context.mouse_right_pressed {
            self.set_default_by_name(&name);
        }
    }
}