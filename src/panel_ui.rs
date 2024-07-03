
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

pub struct TextPanel {
	drag_context: PanelUiDragContext,

	// position: Vector2,
	width: i32,
	height: i32,

	// is_draggable: bool,
	// is_dragging: bool,

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

impl TextPanel {
	// TODO: make more new functions. rust doesn't have defaults so it sucks

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

	#[allow(dead_code)]
	pub fn add_text_buttons_by_d(&mut self, text_array: &[&str], d: &mut RaylibDrawHandle) {
		for text in text_array {
			self.add_text_button_by_d(text, d);
		}
	}
	// TODO Rest of add text buttons

	
	
	// tells me what text button its over and returns it.
	#[allow(dead_code)]
	pub fn get_hovered(&self, mouse_context: &MouseContext) -> Option<&str> {
		let index = self.get_hovered_id(mouse_context)?;
		return Some(&self.text_array[index]);
	}
}

impl PanelLike for TextPanel {
	fn new() -> Self {
		let text_padding = 20;
		Self {
			drag_context: PanelUiDragContext::default(),

			width: text_padding * 2,
			height: text_padding * 2,

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
	fn new_draggable(drag_context: PanelUiDragContext) -> Self {
		Self { drag_context, ..Self::new() }
	}

	fn width(&self)  -> f32 { self.width  as f32 }
	fn height(&self) -> f32 { self.height as f32 }

	fn set_width (&mut self, width : f32) { self.width  = width  as i32 }
	fn set_height(&mut self, height: f32) { self.height = height as i32	}

	fn get_drag_context(&self) -> PanelUiDragContext { self.drag_context }
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext) {
		self.drag_context = drag_context;
	}

	fn get_hovered_id_at(&self, mouse_context: &MouseContext, position: Vector2) -> Option<usize> {
		let mut xx = position.x as i32;
		let mut yy = position.y as i32;

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
	
	// this one doesn't do any recursively, its a 'leaf node'
	fn get_hovered_id_recursively_at(&self, mouse_context: &MouseContext, position: Vector2) -> Vec<usize> {
		self.get_hovered_id_at(mouse_context, position)
			.map(|id| vec![id])
			.unwrap_or_default()
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

// check weather a point is in a rectangle
pub fn point_rec_collision(point: Vector2, rec: Rectangle) -> bool {
	return (rec.x <= point.x && point.x <= rec.x + rec.width)
	    && (rec.y <= point.y && point.y <= rec.y + rec.height);
}








// Panel that has an array of panels, and draws them in a column
// ignores all other positions on child panels
pub struct PanelColumn<T : PanelLike> {
	drag_context: PanelUiDragContext,

	// panel_array: Vec<PanelUi>,
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
	fn new_draggable(drag_context: PanelUiDragContext) -> Self {
		Self { drag_context, ..Self::new() }
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
		for panel in &mut self.panel_array { panel.set_height(height); }
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
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext) {
		self.drag_context = drag_context;
	}

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

	fn draw_panel_at(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2) {
		let mut position = position;
		for panel in self.panel_array.into_iter() {
			let height = panel.height();

			panel.draw_panel_at(d, mouse_context, position);
			position.y += height as f32;
		}
	}
}



pub trait PanelLike {
	// creates a default panel, panel will probably have more in
	// depth versions of these
	fn new() -> Self;
	fn new_draggable(drag_context: PanelUiDragContext) -> Self;

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
	// fn set_position(&mut self) -> Vector2;

	// kinda necessary setter and getter. 
	fn get_drag_context(&self) -> PanelUiDragContext;
	// so we can auto impl some stuff
	fn set_drag_context(&mut self, drag_context: PanelUiDragContext);
	

	// for bounding box purposes
	fn as_rec(&self) -> Rectangle {
		let position = self.get_position();
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
		let rec = Rectangle {
			x: position.x,
			y: position.y,
			..self.as_rec()
		};
		return point_rec_collision(mouse_context.mouse_pos, rec);
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
	fn get_hovered_id_recursively_at(&self, mouse_context: &MouseContext, position: Vector2) -> Vec<usize>;

	// handle mouse dragging
	fn do_dragging   (&mut self, mouse_context: &MouseContext) -> PanelUiDragContext {
		let drag_context = self.do_dragging_at(mouse_context, self.get_drag_context());
		
		self.set_drag_context(drag_context);
		
		return drag_context;
	}
	fn do_dragging_at(&self, mouse_context: &MouseContext, drag_context: PanelUiDragContext) -> PanelUiDragContext;

	// consume self here, should only dray once, clone if required
	fn draw_panel   (self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext)
	where Self : Sized {
		let position = self.get_position();
		self.draw_panel_at(d, mouse_context, position)
	}
	fn draw_panel_at(self, d: &mut RaylibDrawHandle, mouse_context: &MouseContext, position: Vector2);
}