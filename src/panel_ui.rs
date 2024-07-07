
use std::cmp::min;

use raylib::prelude::*;

use crate::MouseContext;

type Vector2 = raylib::math::Vector2;
type Rectangle = raylib::math::Rectangle;

pub trait CanMeasureText {
	fn my_measure_text(&self, text: &str, font_size: i32) -> i32;
}

impl CanMeasureText for RaylibHandle {
	fn my_measure_text(&self, text: &str, font_size: i32) -> i32 {
		self.measure_text(text, font_size)
	}
}

impl CanMeasureText for RaylibDrawHandle<'_> {
	fn my_measure_text(&self, text: &str, font_size: i32) -> i32 {
		self.measure_text(text, font_size)
	}
}

pub trait PanelLike {
	// creates a default panel, panel will probably have more in
	// depth versions of these
	fn new() -> Self where Self: Sized;
	fn new_draggable(drag_context: PanelUiDragContext) -> Self
	where Self : Sized {
		let mut new = Self::new();
		new.set_drag_context(drag_context);
		return new;
	}
	fn new_at_position(position: Vector2) -> Self
	where Self : Sized {
		let mut new = Self::new();
		new.set_position(position);
		new
	}

	fn width (&self) -> f32;
	fn height(&self) -> f32;
	
	// boo oop. but makes some things easier,
	// some panels will panic if you try to do this 
	fn set_width (&mut self, width : f32);
	fn set_height(&mut self, height: f32);

	// meh, wish we could do better than getters
	// and setters, feels too oop
	// any smart compiler would optimize this out, is rust smart? 
	fn get_position(&self) -> Vector2 { self.get_drag_context().position }
	fn set_position(&mut self, position: Vector2) {
		let new_drag = PanelUiDragContext {
			position,
			..self.get_drag_context()
		};
		self.set_drag_context(new_drag);
	}

	// kinda necessary setter and getter. 
	fn get_drag_context(&self) -> PanelUiDragContext;
	// so we can auto impl some stuff
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext);
	

	// for bounding box purposes
	fn as_rec(&self) -> Rectangle { self.as_rec_at(self.get_position()) }
	fn as_rec_at(&self, position: Vector2) -> Rectangle {
		Rectangle {
			x      : position.x,
			y      : position.y,
			width  : self.width(),
			height : self.height(),
		}
	}

	fn mouse_over_panel   (&self, mouse_context: &MouseContext) -> bool {
		self.mouse_over_panel_at(mouse_context, self.get_position())
	}
	fn mouse_over_panel_at(&self, mouse_context: &MouseContext, position: Vector2) -> bool {
		point_rec_collision(mouse_context.mouse_pos, self.as_rec_at(position))
	}
	
	// most if not all panels, should have a list of inner items,
	// that it displays, this function returns the id in the array
	// where it is stored, this might not be the most useful, so
	// panels should also give you some helper functions for this
	fn get_hovered_id   (&self, mouse_context: &MouseContext) -> Option<usize> {
		self.get_hovered_id_at(mouse_context, self.get_position())
	}
	fn get_hovered_id_at(&self, mouse_context: &MouseContext, position: Vector2) -> Option<usize>;

	// this will return an array of id's, user should keep track
	// of the order they entered their items, and this fill allow
	// them to check if the thing they want to check is hovered,
	// this requires panels to only have 1 thing be hovered at a time,
	// witch is true to life as only one thing can be rendered at a time.
	fn get_hovered_id_recursively   (&self, mouse_context: &MouseContext) -> Vec<usize> {
		self.get_hovered_id_recursively_at(mouse_context, self.get_position())
	}
	// this is the base case, for a leaf node panel
	fn get_hovered_id_recursively_at(&self, mouse_context: &MouseContext, position: Vector2) -> Vec<usize> {
		self.get_hovered_id_at(mouse_context, position)
			.map(|id| vec![id])
			.unwrap_or_default()
	}

	// handle mouse dragging
	fn do_dragging   (&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		let drag_context = self.do_dragging_at(mouse_context, self.get_drag_context());
		
		self.set_drag_context(drag_context);
		
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


	// consume self here, should only dray once, clone if required
	fn draw_panel   (&self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext)
	where Self : Sized {
		let position = self.get_position();
		self.draw_panel_at(d, mouse_context, position)
	}
	fn draw_panel_at(&self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2);
}

pub trait DrawableObject {
	fn draw(&self, d: &mut RaylibDrawHandle, rec: Rectangle);
}

impl DrawableObject for dyn PanelLike {
	fn draw(&self, d: &mut RaylibDrawHandle, rec: Rectangle) {
		let position = Vector2 { x: rec.x, y: rec.y };
		// this introduces a bug. mouse pos would be at (0, 0)
		// potentially triggering a on hover thing.
		let mouse_context = MouseContext::default();
		self.draw_panel_at(d, &mouse_context, position);
	}
}

impl DrawableObject for Color {
	fn draw(&self, d: &mut RaylibDrawHandle, rec: Rectangle) {
		d.draw_rectangle_rec(rec, self)
	}
}

impl DrawableObject for &Texture2D {
	fn draw(&self, d: &mut RaylibDrawHandle, rec: Rectangle) {
		// floats might screw us
		assert!(self.width  == rec.width  as i32);
		assert!(self.height == rec.height as i32);
		// hope you smart enough for this, texture needs to be the right size
		d.draw_texture(self, rec.x as i32, rec.y as i32, Color::WHITE)
	}
}

// TODO: make this accept a drawable object?
// the only two objects i have in mind are text and images
// i could even just impl those cases in this file


#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct PanelUiDragContext {
	pub is_draggable: bool, // because were going to be passing this around a lot

	pub position: Vector2,
	pub is_dragging: bool,
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

#[derive(Debug)]
pub struct TextPanel {
	drag_context: PanelUiDragContext,

	width: i32,
	height: i32,

	text_array: Vec<String>,
	text_width_array: Vec<i32>,
	text_height_array: Vec<i32>,

	// if your going to change these, do it immediately
	pub default_text_height: i32,
	pub text_padding: i32,

	pub item_padding: i32,

	pub background_color: Color,
	pub text_color: Color,
	pub hover_color: Option<Color>, // TODO: make optional
}

impl TextPanel {
	// TODO: make more new functions. rust doesn't have defaults so it sucks
	pub fn new_custom(text_height: i32, text_padding: i32, item_padding: i32, background_color: Color, text_color: Color, hover_color: Option<Color>) -> Self {
		Self {
			default_text_height: text_height,
			text_padding,
			item_padding,
			background_color,
			text_color,
			hover_color,

			height: text_padding * 2,

			..Self::new()
		}
	}

	pub fn add_text_button(&mut self, text: &str, rl: &mut impl CanMeasureText) {
		self.add_text_button_ex(text, rl.my_measure_text(text, self.default_text_height), self.default_text_height)
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

	pub fn add_text_buttons(&mut self, text_array: &[&str], rl: &mut impl CanMeasureText) {
		for text in text_array {
			self.add_text_button(text, rl);
		}
	}
	
	
	// tells me what text button its over and returns it.
	#[allow(dead_code)]
	pub fn get_hovered(&self, mouse_context: &MouseContext) -> Option<&str> {
		let index = self.get_hovered_id(mouse_context)?;
		return Some(&self.text_array[index]);
	}
}

impl PanelLike for TextPanel {
	fn new() -> Self {
		const TEXT_PADDING: i32 = 20;
		Self {
			drag_context: PanelUiDragContext::default(),

			width: 0,
			height: TEXT_PADDING * 2,

			default_text_height: 20,
			text_padding: TEXT_PADDING,

			item_padding: 5,

			text_array: vec![],
			text_width_array: vec![],
			text_height_array: vec![],

			// Just some random colors
			background_color: Color::YELLOW,
			text_color: Color::RED,
			hover_color: Some(Color::ORANGE),
		}
	}

	fn width(&self)  -> f32 { self.width  as f32 }
	fn height(&self) -> f32 { self.height as f32 }

	fn set_width (&mut self, width : f32) { self.width  = width  as i32 }
	fn set_height(&mut self, height: f32) { self.height = height as i32	}

	fn get_drag_context(&self) -> PanelUiDragContext                 { self.drag_context }
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext) { self.drag_context = drag_context; }

	fn get_hovered_id_at(&self, mouse_context: &MouseContext, position: Vector2) -> Option<usize> {
		let mut xx = position.x as i32;
		let mut yy = position.y as i32;

		xx += self.text_padding; // indent for text
		yy += self.text_padding; // indent for backing padding

		for i in 0..self.text_array.len() {
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

			yy += height + self.item_padding;
		}

		return None;
	}
	
	fn draw_panel_at(&self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		// TODO: just as rec?

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

			if let Some(hover_color) = self.hover_color {
				if point_rec_collision(mouse_context.mouse_pos, rec) {
					let padded = pad_rectangle_ex(rec, (self.text_padding / 2) as f32, (self.text_padding / 2) as f32, 0.0, 0.0);
					d.draw_rectangle_rec(padded, hover_color);
				}
			}

			// TODO: center text by some sort of anchor
			d.draw_text(&text, xx, yy, height, self.text_color);
			yy += height;
			yy += self.item_padding;
		}
	}
}


// check weather a point is in a rectangle
pub fn point_rec_collision(point: Vector2, rec: Rectangle) -> bool {
	return (rec.x <= point.x && point.x <= rec.x + rec.width )
	    && (rec.y <= point.y && point.y <= rec.y + rec.height);
}

pub fn pad_rectangle_ex(rec: Rectangle, left: f32, right: f32, top: f32, bottom: f32) -> Rectangle {
    Rectangle {
        x:      rec.x      - left,
        y:      rec.y      - top,
        width:  rec.width  + right * 2.0,
        height: rec.height + bottom * 2.0,
    }
}
pub fn pad_rectangle(rec: Rectangle, padding: f32) -> Rectangle {
    pad_rectangle_ex(rec, padding, padding, padding, padding)
}


// Panel that has an array of panels, and draws them in a column
// ignores all other positions on child panels
pub struct PanelColumn<T : PanelLike> {
	drag_context: PanelUiDragContext,

	panel_array: Vec<T>,
	panel_draggable_array: Vec<bool>,
}

impl<T : PanelLike> PanelColumn<T> {
	pub fn add_panel(&mut self, new_panel: T, is_draggable: bool) {		
		self.panel_array.push(new_panel);
		self.panel_draggable_array.push(is_draggable);
		
		self.equalize_widths()
	}

	// this widths the larges panel width, and sets all panels widths appropriately
	fn equalize_widths(&mut self) {
		let max_width = self.width();

		for panel in &mut self.panel_array {
			panel.set_width(max_width);
		}
	}
}

impl<T : PanelLike> PanelLike for PanelColumn<T> {
	fn new() -> Self {
		Self {
			drag_context: PanelUiDragContext::default(),
			panel_array: vec![],
			panel_draggable_array: vec![],
		}
	}

	fn width(&self)  -> f32 {
		self.panel_array
			.iter()
			.map(|panel| panel.width())
			.fold(None, |z, u| {
				if let Some(z) = z {
					Some((u > z).then_some(u).unwrap_or(z))
				} else { Some(u) }
			})
			.unwrap_or(0.0)
	}
	fn height(&self) -> f32 {
		self.panel_array.iter().map(|panel| panel.height()).sum()
	}

	fn set_width (&mut self, width : f32) {
		for panel in &mut self.panel_array { panel.set_width (width ); }
	}
	fn set_height(&mut self, height: f32) {
		let current_height = self.height();
		// Scale all children to add up to height
		for panel in &mut self.panel_array {
			panel.set_height(panel.height() / current_height * height);
		}
	}

	fn get_hovered_id_at(&self, mouse_context: &MouseContext, position: Vector2) -> Option<usize> {
		let mut position = position;
		for (i, panel) in self.panel_array.iter().enumerate() {
			if panel.mouse_over_panel_at(mouse_context, position) {
				return Some(i);
			}
			position.y += panel.height();
		}

		return None;
	}
	fn get_hovered_id_recursively_at(&self, mouse_context: &MouseContext, mut position: Vector2) -> Vec<usize> {
		self.get_hovered_id_at(mouse_context, position)
			.map(|id| {
				(0..id)
					.map(|i| &self.panel_array[i])
					.for_each(|panel| position.y += panel.height());

				[
					vec![id],
					self.panel_array[id].get_hovered_id_recursively_at(mouse_context, position),
				].concat()

			})
			.unwrap_or_default()
	}

	fn get_drag_context(&self) -> PanelUiDragContext { self.drag_context }
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext) { self.drag_context = drag_context; }

	fn do_dragging_at(&self, mouse_context: &MouseContext, mut drag_context: PanelUiDragContext) -> PanelUiDragContext {
		if drag_context.is_draggable == false {
			panic!("Panel isn't draggable")
		}
	
		let mut position = drag_context.position;
		
		for i in 0..self.panel_array.len() {
			let panel = &self.panel_array[i];
			
			assert!(panel.get_drag_context().is_draggable == false); // this panel doesn't handle this
			
			let tmp      = position;
			position.y           += panel.height();
			let position = tmp;
			
			if !self.panel_draggable_array[i] { continue; }
	
			let panel_drag_context = PanelUiDragContext {
				position,
				..drag_context
			};

			let new_drag = panel.do_dragging_at(mouse_context, panel_drag_context);

			// I don't think this is technically correct, but, it 'fails correctly'
			// the reason for the double drag is because new drag is false, and it still
			// passes the old drag to the next function. this could be explained better 
			if new_drag != panel_drag_context {
				drag_context.position += new_drag.position - panel_drag_context.position;
				drag_context.is_dragging = new_drag.is_dragging;

				break; // dragging does one and a half times as fast. lol
			}
		}
	
		return drag_context;
	}

	fn draw_panel_at(&self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		let mut position = position;
		for panel in self.panel_array.iter() {
			let height = panel.height();

			panel.draw_panel_at(d, mouse_context, position);
			position.y += height as f32;
		}
	}
}


pub struct GridDrawPanel<T : DrawableObject> {
	drag_context: PanelUiDragContext,

	// true if by cols, false if by rows
	by_cols: bool,
	// this controls how many items in a row or col before
	// moving onto the next row/col. cannot be 0
	run_length: usize,

	grid_array: Vec<T>,
	highlights_array: Vec<Vec<Color>>,

	// remove pub?
	pub item_width  : i32,
	pub item_height : i32,

	pub item_padding: i32,

	pub highlight_color  : Option<Color>,
	pub background_color : Option<Color>,
}

impl<T : DrawableObject> GridDrawPanel<T>  {
	pub fn new_custom(
		position: Vector2,
		by_cols: bool, run_length: usize,
		item_width: i32, item_height: i32, item_padding: i32,
		highlight_color: Option<Color>, background_color: Option<Color>
	) -> Self {
		Self {
			by_cols, run_length,
			item_width, item_height, item_padding,
			highlight_color, background_color,
			..Self::new_at_position(position)
		}
	}

	pub fn add(&mut self, new_object: T) {
		self.grid_array.push(new_object);
		self.highlights_array.push(vec![]);
	}

	pub fn add_with_highlight(&mut self, new_object: T, highlight_array: &[Color]) {
		self.grid_array.push(new_object);
		self.highlights_array.push(highlight_array.to_vec());
	}

	fn length_helper(&self, by_cols: bool, length: i32) -> i32 {
		if self.grid_array.len() == 0 { return 0; }
		let num_wide = if by_cols {
			min(self.grid_array.len(), self.run_length) as i32
		} else {
			self.grid_array.len().div_ceil(   self.run_length) as i32
		};
		length * num_wide + self.item_padding * (num_wide - 1)
	}

	// TODO: Clean this up
	fn position_of_item_at(&self, index: usize, position: Vector2) -> Vector2 {
		let (move_by, next_cycle) = if self.by_cols {
			(
				Vector2 { x: 0.0, y: (self.item_height + self.item_padding) as f32},
				Vector2 { x: (self.item_width + self.item_padding) as f32, y: 0.0},
			)
		} else {
			(
				Vector2 { x: (self.item_width + self.item_padding) as f32, y: 0.0},
				Vector2 { x: 0.0, y: (self.item_height + self.item_padding) as f32},
			)
		};

		position
			+ (move_by   .scale_by((index / self.run_length) as f32))
			+ (next_cycle.scale_by((index % self.run_length) as f32))
	}
	
	fn rec_of_item_at(&self, index: usize, position: Vector2) -> Rectangle {
		let position = self.position_of_item_at(index, position);
		Rectangle {
			x: position.x,
			y: position.y,
			width  : self.item_width  as f32,
			height : self.item_height as f32,
		}
	}

	fn draw_highlights(&self, d: &mut RaylibDrawHandle, rec: Rectangle, colors: &[Color]) {
		if colors.len() == 0 { return; }

		// backing color
		let rec = pad_rectangle(rec, (self.item_padding / 2) as f32);
		d.draw_rectangle_rec(rec, colors[0]);

		if colors.len() == 1 { return; }

		{ // to the left
			let mut rec = rec;
			rec.width /= 2.0;
			d.draw_rectangle_rec(rec, colors[1]);
		}

		if colors.len() == 2 { return; }

		// cool middle stripes
		let top_stripe = Rectangle {
			x: rec.x + rec.width / 3.0,  y: rec.y,
			width: rec.width / 3.0,     height: rec.height,
		};
		let middle_stripe = Rectangle {
			x: rec.x,              y: rec.y + rec.height / 3.0,
			width: rec.width, height: rec.height / 3.0,
		};
		d.draw_rectangle_rec(top_stripe, colors[2]);
		d.draw_rectangle_rec(middle_stripe, colors[2]);

		if colors.len() > 3 { panic!("Currently do not handle more than 3 colors in panel highlight") }
	}
}

impl<T : DrawableObject> PanelLike for GridDrawPanel<T> {
	fn new() -> Self {
		Self {
			drag_context     : PanelUiDragContext::default(),
			
			by_cols          : true,
			run_length       : 5,
			
			grid_array       : vec![],
			highlights_array : vec![],
			
			item_width       : 64,
			item_height      : 64,
			item_padding     : 10,
			
			highlight_color  : Some(Color::ORANGE),
			background_color : None,
		}
	}

	fn width (&self) -> f32 { self.length_helper(self.by_cols, self.item_width) as f32 }
	fn height(&self) -> f32 { self.length_helper(!self.by_cols, self.item_height) as f32 }

	// TODO: change item padding to fit into new width. for pros only
	fn set_width (&mut self, _width : f32) { todo!() }
	fn set_height(&mut self, _height: f32) { todo!() }

	fn get_drag_context(&self) -> PanelUiDragContext                 { self.drag_context }
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext) { self.drag_context = drag_context; }

	fn get_hovered_id_at(&self, mouse_context: &MouseContext, position: Vector2) -> Option<usize> {
		for i in 0..self.grid_array.len() {
			let rec = self.rec_of_item_at(i, position);

			if point_rec_collision(mouse_context.mouse_pos, rec) {
				return Some(i);
			}
		}
		return None;
	}

	// maybe when we upgrade this
	// fn get_hovered_id_recursively_at(&self, mouse_context: &MouseContext, position: Vector2) -> Vec<usize>;

	fn draw_panel_at(&self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		if let Some(color) = self.background_color {
			d.draw_rectangle_rec(self.as_rec_at(position), color);
		}

		let hovered = self.get_hovered_id_at(mouse_context, position);

		for (i, drawable) in self.grid_array.iter().enumerate() {
			let rec = self.rec_of_item_at(i, position);

			let mut highlights = self.highlights_array[i].clone();
			if let Some(highlight_color) = self.highlight_color {
				if hovered == Some(i) {
					highlights.push(highlight_color);
				}
			}

			self.draw_highlights(d, rec, &highlights);

			drawable.draw(d, rec);
		}
	}
}
