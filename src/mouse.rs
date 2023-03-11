type MousePosition = glutin::dpi::PhysicalPosition<f64>;

#[derive(Debug, Copy, Clone)]
pub struct MouseState {
    pub left_mouse_button_down: bool,
    pub right_mouse_button_down: bool,
    pub current_position: Option<MousePosition>,
    pub previous_position: Option<MousePosition>,
    pub scroll_delta: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState {
            left_mouse_button_down: false,
            right_mouse_button_down: false,
            current_position: None,
            previous_position: None,
            scroll_delta: 0.0,
        }
    }

    pub fn position_delta(&mut self) -> MousePosition {
        self.current_position.zip(self.previous_position).map_or(
            MousePosition::new(0.0, 0.0),
            |(current_position, previous_position)| {
                MousePosition::new(
                    current_position.x - previous_position.x,
                    current_position.y - previous_position.y,
                )
            },
        )
    }

    pub fn handle_window_event(&mut self, event: &glutin::event::WindowEvent) {
        use glutin::event::{ElementState, MouseButton, WindowEvent};

        match event {
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.left_mouse_button_down = true,

                (ElementState::Released, MouseButton::Left) => self.left_mouse_button_down = false,
                (ElementState::Pressed, MouseButton::Right) => self.right_mouse_button_down = true,
                (ElementState::Released, MouseButton::Right) => {
                    self.right_mouse_button_down = false
                }
                _ => {}
            },
            WindowEvent::CursorLeft { .. } => {
                self.left_mouse_button_down = false;
                self.right_mouse_button_down = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.previous_position = self.current_position;
                self.current_position = Some(*position);
            }
            _ => {}
        }
    }
}
