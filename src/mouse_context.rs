use raylib::prelude::*;

#[derive(Debug, Default)]
pub struct MouseContext {
    pub mouse_pos   :  Vector2,
    pub mouse_delta :  Vector2,
    pub mouse_left_pressed  : bool,
    pub mouse_left_released : bool,
    pub mouse_right_pressed : bool,

	// TODO: Get key presses in here

	// TODO remove
    pub over_file_dialog : bool,
}

impl MouseContext {
	pub fn make_context(rl: &RaylibHandle) -> Self {
		MouseContext {
            mouse_pos           : rl.get_mouse_position(),
            mouse_delta         : rl.get_mouse_delta(),
            mouse_left_pressed  : rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            mouse_left_released : rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
            mouse_right_pressed : rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT),
    
            over_file_dialog : false,
        }
	}
}