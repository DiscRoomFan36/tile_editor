
use raylib::prelude::*;

// TODO: make this accept a drawable object?
// the only two objects i have in mind are text and images
// i could even just impl those cases in this file

use crate::{pad_rectangle_ex, MouseContext};

type Vector2 = raylib::math::Vector2;
type Rectangle = raylib::math::Rectangle;

// Lets think here, do i want to have a full context thing
// that you make a thing out of, maybe, but that feels a little bad
// for some reason. alternately, the programmer is going to make there own state
// and if they want a menu then they just make their menu, maybe by a function
// or a method, and they can store their own dam drag context. its their
// problem

#[derive(Debug)]
pub struct PanelUi {
	position: Vector2,
	width: i32,
	height: i32,

	is_draggable: bool,
	is_dragging: bool,

	default_text_height: i32,
	text_padding: i32,

	item_padding: i32,

	text_array: Vec<String>,
	text_width_array: Vec<i32>,
	text_height_array: Vec<i32>,

	// TODO: remove
	pub background_color: Color,
	text_color: Color,
	hover_color: Color, // TODO: make optional
}

// add text to thing
// when asked for, tell if mouse is over something

impl PanelUi {
	// TODO: accept more arguments, or not. rust doesn't have defaults so it sucks
	pub fn new(position: Vector2, text_padding: i32) -> Self {
		PanelUi {
			position,
			width: text_padding * 2,
			height: text_padding * 2,

			is_draggable: false,
			is_dragging: false,

			default_text_height: 20,
			text_padding,

			item_padding: 5,

			text_array: vec![],
			text_width_array: vec![],
			text_height_array: vec![],

			background_color: Color::YELLOW,
			text_color: Color::RED,
			hover_color: Color::ORANGE,
		}
	}

	pub fn new_draggable(text_padding: i32, dragging_context: PanelUiDragContext) -> Self {
		PanelUi {
			position: dragging_context.position,
			is_draggable: true,
			is_dragging: dragging_context.is_dragging,
			..Self::new(dragging_context.position, text_padding)
		}
	}

	pub fn get_drag_context(&self) -> PanelUiDragContext {
		PanelUiDragContext {
			position: self.position,
			is_dragging: self.is_dragging,
		}
	}

	pub fn as_rec(&self) -> Rectangle {
		Rectangle {
			x: self.position.x,
			y: self.position.y,
			width: self.width as f32,
			height: self.height as f32,
		}
	}
	
	#[allow(dead_code)]
	pub fn add_text_button(&mut self, text: &str, text_width: i32) {
		self.add_text_button_ex(text, text_width, self.default_text_height)
	}
	pub fn add_text_button_by_d(&mut self, text: &str, d: &mut RaylibDrawHandle) {
		self.add_text_button_ex(text, d.measure_text(text, self.default_text_height), self.default_text_height)
	}
	#[allow(dead_code)]
	pub fn add_text_button_by_rl(&mut self, text: &str, rl: &mut RaylibHandle) {
		self.add_text_button_ex(text, rl.measure_text(text, self.default_text_height), self.default_text_height)
	}

	pub fn add_text_button_ex(&mut self, text: &str, text_width: i32, text_height: i32) {
		self.text_array       .push(text.to_string());
		self.text_width_array .push(text_width);
		self.text_height_array.push(text_height);

		let with_pad = text_width + self.text_padding * 2;
		if self.width < with_pad {
			self.width = with_pad;
		}
	
		self.height += text_height;

		// put some padding in between items
		if self.text_array.len() > 1 {
			self.height += self.item_padding;
		}
	}

	pub fn add_text_buttons_by_d(&mut self, text_array: &[&str], d: &mut RaylibDrawHandle) {
		for text in text_array {
			self.add_text_button_by_d(text, d);
		}
	}
	// TODO Rest

	#[allow(dead_code)]
	pub fn mouse_over_panel(&self, mouse_context: &MouseContext) -> bool {
		point_rec_collision(mouse_context.mouse_pos, self.as_rec())
	}
	
	// TODO: do something to help with dragging
	// TODO: need more time to do this, need to pass in extra info for this 
	pub fn do_dragging(&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		if self.is_draggable == false {
			panic!("Panel isn't draggable by default, use different constructor")
		}

		if self.mouse_over_panel(mouse_context) && mouse_context.mouse_left_pressed {
			self.is_dragging = true;
		}

		if self.is_dragging {
			self.position += mouse_context.mouse_delta;
			if mouse_context.mouse_left_released {
				self.is_dragging = false;
			}
		}

		return self.get_drag_context();
	}
	
	#[allow(dead_code)]
	pub fn get_hovered_id(&self, mouse_context: &MouseContext) -> Option<usize> {
		let mut xx = self.position.x as i32;
		let mut yy = self.position.y as i32;

		xx += self.text_padding; // indent for text
		yy += self.text_padding; // indent for backing padding

		for i in 0..self.text_array.len() {
			// let text = &self.text_array[i];
			// let width = self.text_width_array[i];
			let height = self.text_height_array[i];
		
			let rec = Rectangle {
				x: xx as f32, y: yy as f32,
				width: (self.width - self.text_padding * 2) as f32,
				height: height as f32
			};

			if point_rec_collision(mouse_context.mouse_pos, rec) {
				// we found it. maybe do some checking to assert that were not above anything else?
				return Some(i);
			}

			yy += height;

			// for the item padding
			yy += self.item_padding
		}

		return None;
	}
	// tells me what text button its over and returns it.
	#[allow(dead_code)]
	pub fn get_hovered(&self, mouse_context: &MouseContext) -> Option<&str> {
		let index = self.get_hovered_id(mouse_context)?;

		return Some(&self.text_array[index]);
	}

	// consume self here, should only dray once, clone if required
	pub fn draw_panel(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext) {
		let mut xx = self.position.x as i32;
		let mut yy = self.position.y as i32;

		d.draw_rectangle(xx, yy, self.width, self.height, self.background_color);

		xx += self.text_padding;
		yy += self.text_padding;

		// for (text, width, height) in self.text_array.iter() {
		for i in 0..self.text_array.len() {
			let text = &self.text_array[i];
			// let width = self.text_width_array[i];
			let height = self.text_height_array[i];

			let rec = Rectangle {
				x: xx as f32,
				y: yy as f32,
				width: (self.width - self.text_padding * 2) as f32, // this width is the bounding box, it is meant to be self.width
				height: height as f32
			};

			if point_rec_collision(mouse_context.mouse_pos, rec) {
				let padded = pad_rectangle_ex(rec, (self.text_padding / 2) as f32, (self.text_padding / 2) as f32, 0.0, 0.0);
				d.draw_rectangle_rec(padded, self.hover_color);
			}

			d.draw_text(&text, xx, yy, height, self.text_color);
			yy += height;
			yy += self.item_padding;
		}
	}
}

// maybe have text padding?
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PanelUiDragContext {
	position: Vector2,
	is_dragging: bool,
}

impl PanelUiDragContext {
	pub fn new(position: Vector2) -> Self {
		Self {
			position,
			..Self::default()
		}
	}
}

// check weather a point is in a rectangle
pub fn point_rec_collision(point: Vector2, rec: Rectangle) -> bool {
	return (rec.x <= point.x && point.x <= rec.x + rec.width)
	    && (rec.y <= point.y && point.y <= rec.y + rec.height);
}








// Panel that has an array of panels, and draws them
// (on top of each other?)

pub struct PanelPanel {
	position: Vector2, // panel has its own position, and all child panels are relative coordinates

	is_draggable: bool,
	is_dragging: bool,

	panel_array: Vec<PanelUi>,
	panel_draggable_array: Vec<bool>,
}

impl PanelPanel {
	pub fn new(position: Vector2) -> Self {
		Self {
			position,

			is_draggable: false,
			is_dragging: false,

			panel_array: vec![],
			panel_draggable_array: vec![],
		}
	}
	pub fn new_draggable(dragging_context: PanelUiDragContext) -> Self {
		Self {
			is_draggable: true,
			is_dragging: dragging_context.is_dragging,
			..Self::new(dragging_context.position)
		}
	}

	pub fn get_drag_context(&self) -> PanelUiDragContext {
		PanelUiDragContext {
			position: self.position,
			is_dragging: self.is_dragging,
		}
	}

	pub fn add_panel(&mut self, mut new_panel: PanelUi, is_draggable: bool) {
		self.panel_array.push(new_panel);
		self.panel_draggable_array.push(is_draggable);
	}

	// this widths the larges panel width, and sets all panels widths appropriately
	pub fn equalize_widths(&mut self) {
		// maybe just return? for now it will tell me if im being stupid
		assert!(self.panel_array.len() > 0, "You are dumb");

		let max_width = self.panel_array
			.iter()
			.map(|panel| panel.width)
			.max()
			.unwrap();

		for panel in &mut self.panel_array {
			panel.width = max_width;
		}
	}

	// this finds the things that are draggable and calls do_dragging on them
	// this needs to do something with a context
	// TODO

	// this panel panel will return a drag context that is for itself,
	// so that any part that drags, will drag the whole panel,
	// need to make another thing that dose sort of the same thing, but 
	// is more of a window in window deal
	pub fn do_dragging(&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		assert!(self.is_draggable);

		let mut yy = 0.0;
		
		for i in 0..self.panel_array.len() {
			let panel = &mut self.panel_array[i];
			
			assert!(panel.is_draggable == false); // this might get weird. soft todo
			
			let tmp = yy;
			yy          += panel.height as f32;
			let yy  = tmp;

			if !self.panel_draggable_array[i] { continue; }
			
			

			let tmp = panel.position;
			panel.position += self.position;
			panel.position.y += yy;

			// duplicate code.
			// maybe make a function on PanelUi that accepts a DragContext,
			// and returns a new one, because all we really need from it is its width
			if panel.mouse_over_panel(mouse_context) && mouse_context.mouse_left_pressed {
				self.is_dragging = true;
			}

			if self.is_dragging {
				self.position += mouse_context.mouse_delta;
				if mouse_context.mouse_left_released {
					self.is_dragging = false;
				}
				panel.position = tmp;
				break;
			}

			panel.position = tmp;
		}

		// TODO: handle panels that are draggable in there own way.

		return self.get_drag_context();
	}

	// hmmm it doesn't make sense to add stuff to the start
	// position here, if were just going to try and make
	// them into a list of sorts and put them one below the other.
	// @Cleanup?
	pub fn draw_panel(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext) {
		let mut yy = self.position.y as i32;
		for mut panel in self.panel_array.into_iter() {
			panel.position.x += self.position.x;
			panel.position.y += yy as f32;
			
			yy += panel.height;

			panel.draw_panel(d, mouse_context);
		}
	}
}


