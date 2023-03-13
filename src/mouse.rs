type MousePosition = glutin::dpi::PhysicalPosition<f64>;

#[derive(Debug, Copy, Clone)]
pub struct MouseState {
    left_button_down: bool,
    right_button_down: bool,
    middle_button_down: bool,
    current_position: Option<MousePosition>,
    previous_position: Option<MousePosition>,
    scroll_delta: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState {
            left_button_down: false,
            right_button_down: false,
            middle_button_down: false,
            current_position: None,
            previous_position: None,
            scroll_delta: 0.0,
        }
    }

    pub fn is_left_button_down(&self) -> bool {
        self.left_button_down
    }

    pub fn is_right_button_down(&self) -> bool {
        self.right_button_down
    }

    pub fn is_middle_button_down(&self) -> bool {
        self.middle_button_down
    }

    pub fn position_delta(&mut self) -> MousePosition {
        self.previous_position
            .take()
            .zip(self.current_position)
            .map_or(MousePosition::new(0.0, 0.0), |(previous, current)| {
                MousePosition::new(current.x - previous.x, current.y - previous.y)
            })
    }

    pub fn position(&self) -> Option<MousePosition> {
        self.current_position
    }

    pub fn scroll_delta(&mut self) -> f32 {
        let last_value = self.scroll_delta;
        self.scroll_delta = 0.0;
        last_value
    }

    pub fn handle_window_event(&mut self, event: &glutin::event::WindowEvent) {
        use glutin::event::{ElementState, MouseButton, WindowEvent};

        match event {
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.left_button_down = true,
                (ElementState::Released, MouseButton::Left) => self.left_button_down = false,
                (ElementState::Pressed, MouseButton::Right) => self.right_button_down = true,
                (ElementState::Released, MouseButton::Right) => self.right_button_down = false,
                (ElementState::Pressed, MouseButton::Middle) => self.middle_button_down = true,
                (ElementState::Released, MouseButton::Middle) => self.middle_button_down = false,
                _ => {}
            },
            WindowEvent::CursorLeft { .. } => {
                self.left_button_down = false;
                self.right_button_down = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.previous_position = self.current_position;
                self.current_position = Some(*position);
            }
            WindowEvent::MouseWheel {
                delta: glutin::event::MouseScrollDelta::LineDelta(_, delta),
                ..
            } => {
                self.scroll_delta = *delta;
            }
            _ => {}
        }
    }
}
