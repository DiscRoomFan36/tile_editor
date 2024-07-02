
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

	#[allow(dead_code)]
	pub fn new_draggable(text_padding: i32, dragging_context: PanelUiDragContext) -> Self {
		PanelUi {
			position: dragging_context.position,
			is_draggable: true,
			is_dragging: dragging_context.is_dragging,
			..Self::new(dragging_context.position, text_padding)
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
	// TODO Rest of add text buttons

	
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


}

impl PanelLike for PanelUi {
	fn as_rec(&self) -> Rectangle {
		Rectangle {
			x: self.position.x,
			y: self.position.y,
			width: self.width as f32,
			height: self.height as f32,
		}
	}

	fn get_drag_context(&self) -> PanelUiDragContext {
		PanelUiDragContext {
			is_draggable: self.is_draggable,
			position: self.position,
			is_dragging: self.is_dragging,
		}
	}

	fn do_dragging(&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		let drag_context = self.do_dragging_at(mouse_context, self.get_drag_context());

		self.position = drag_context.position;
		self.is_dragging = drag_context.is_dragging;

		return drag_context;
	}
	fn do_dragging_at(&self, mouse_context: &MouseContext, mut drag_context: PanelUiDragContext) -> PanelUiDragContext {
		if drag_context.is_draggable == false {
			panic!("Panel isn't draggable")
		}

		if self.mouse_over_panel_at(mouse_context, drag_context.position) && mouse_context.mouse_left_pressed {
			drag_context.is_dragging = true;
		}

		if drag_context.is_dragging {
			drag_context.position += mouse_context.mouse_delta;
			if mouse_context.mouse_left_released {
				drag_context.is_dragging = false;
			}
		}

		return drag_context;
	}


	// consume self here, should only draw once, clone if required
	fn draw_panel(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext) {
		let position = self.position;
		self.draw_panel_at(d, mouse_context, position)
	}
	fn draw_panel_at(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		let mut xx = position.x as i32;
		let mut yy = position.y as i32;

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

			// TODO: center text by some sort of anchor
			d.draw_text(&text, xx, yy, height, self.text_color);
			yy += height;
			yy += self.item_padding;
		}
	}
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PanelUiDragContext {
	is_draggable: bool, // because were going to be passing this around a lot

	position: Vector2,
	is_dragging: bool,
}

impl PanelUiDragContext {
	pub fn new(position: Vector2) -> Self {
		Self {
			is_draggable: true, // if your making this yourself, probably
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

	pub fn add_panel(&mut self, new_panel: PanelUi, is_draggable: bool) {
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
}

impl PanelLike for PanelPanel {
	fn as_rec(&self) -> Rectangle {
		Rectangle {
			x: self.position.x,
			y: self.position.y,
			width : self.panel_array.iter().map(|panel| panel.width ).max().unwrap_or(0) as f32,
			height: self.panel_array.iter().map(|panel| panel.height).sum::<i32>() as f32,
		}
	}

	fn get_drag_context(&self) -> PanelUiDragContext {
		PanelUiDragContext {
			is_draggable: self.is_draggable,
			position: self.position,
			is_dragging: self.is_dragging,
		}
	}

	fn do_dragging(&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		let drag_context = self.do_dragging_at(mouse_context, self.get_drag_context());

		self.position = drag_context.position;
		self.is_dragging = drag_context.is_dragging;

		return drag_context;
	}
	fn do_dragging_at(&self, mouse_context: &MouseContext, mut drag_context: PanelUiDragContext) -> PanelUiDragContext {
		if drag_context.is_draggable == false {
			panic!("Panel isn't draggable")
		}
	
		let mut position = drag_context.position;
		
		for i in 0..self.panel_array.len() {
			let panel = &self.panel_array[i];
			
			assert!(panel.is_draggable == false); // this panel doesn't handle this
			
			let tmp      = position;
			position.y           += panel.height as f32;
			
			if !self.panel_draggable_array[i] { continue; }
			
			let position = tmp;
	
			let panel_drag_context = PanelUiDragContext {
				is_draggable: drag_context.is_draggable,
				position,
				is_dragging: drag_context.is_dragging,
			};

			let new_drag = panel.do_dragging_at(mouse_context, panel_drag_context.clone());

			// I don't think this is technically correct, but, it 'fails correctly'
			// the reason for the double drag is because new drag is false, and it still
			// passes the old drag to the next function. this could be explained better 
			if new_drag != panel_drag_context {
				drag_context.position += new_drag.position - panel_drag_context.position;
				drag_context.is_dragging = new_drag.is_dragging;

				break; // dragging does twice as fast when you don't break?
			}
		}
	
		return drag_context;
	}

	fn draw_panel(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext) {
		let position = self.position;
		self.draw_panel_at(d, mouse_context, position)
	}
	fn draw_panel_at(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		let mut position = position;
		for panel in self.panel_array.into_iter() {
			let height = panel.height;

			panel.draw_panel_at(d, mouse_context, position);
			position.y += height as f32;
		}
	}
}



pub trait PanelLike {

	// meh, wish we could do better than setters
	// and getters, feels too oop
	// any smart compiler would optimize this out, is rust smart? 
	fn get_position(&self) -> Vector2 {
		self.get_drag_context().position
	}
	// fn set_position(&mut self) -> Vector2;

	// some new function

	// some add function

	// for bounding box purposes
	fn as_rec(&self) -> Rectangle;

	fn get_drag_context(&self) -> PanelUiDragContext;

	fn mouse_over_panel(&self, mouse_context: &MouseContext) -> bool {
		self.mouse_over_panel_at(mouse_context, self.get_position())
	}
	fn mouse_over_panel_at(&self, mouse_context: &MouseContext, position: Vector2) -> bool {
		let rec = Rectangle {
    		x: position.x,
			y: position.y,
			..self.as_rec()
		};

		return point_rec_collision(mouse_context.mouse_pos, rec);
	}
	
	// handle mouse dragging
	fn do_dragging(&mut self, mouse_context: &MouseContext) -> PanelUiDragContext;
	fn do_dragging_at(&self, mouse_context: &MouseContext, drag_context: PanelUiDragContext) -> PanelUiDragContext;

	// consume self here, should only dray once, clone if required
	fn draw_panel(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext);
	fn draw_panel_at(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2);
}

