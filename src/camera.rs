use crate::{math::affine::transforms, mouse::MouseState, window::Window};
use glutin::dpi::PhysicalPosition;
use nalgebra::{Matrix4, Vector3, Vector4};

#[derive(Debug)]
pub struct Camera {
    pub azimuth: f32,
    pub altitude: f32,
    pub distance: f32,
    pub center: Vector3<f32>,
}

impl Camera {
    const ROTATION_SPEED: f32 = 0.05;
    const MOVEMENT_SPEED: f32 = 0.01;

    pub fn new() -> Camera {
        Camera {
            azimuth: 0.0,
            altitude: 0.0,
            distance: 1.0,
            center: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn update_from_mouse(&mut self, mouse: &mut MouseState, window: &Window) {
        let mouse_delta = mouse.position_delta();

        if !window.imgui_using_mouse() {
            self.update_angles(mouse, &mouse_delta);
            self.update_center(mouse, &mouse_delta);

            self.distance -= mouse.scroll_delta();

            if self.distance < 0.0 {
                self.distance = 0.0;
            }
        }
    }

    fn update_angles(&mut self, mouse: &MouseState, mouse_delta: &PhysicalPosition<f64>) {
        if mouse.is_left_button_down() {
            self.azimuth += mouse_delta.x as f32 * Self::ROTATION_SPEED;
            self.altitude += mouse_delta.y as f32 * Self::ROTATION_SPEED;
        }
    }

    fn update_center(&mut self, mouse: &MouseState, mouse_delta: &PhysicalPosition<f64>) {
        if mouse.is_right_button_down() {
            self.center += (transforms::rotate_y(-self.azimuth)
                * transforms::rotate_x(-self.altitude)
                * Vector4::new(-mouse_delta.x as f32, mouse_delta.y as f32, 0.0, 0.0))
            .xyz()
                * self.distance
                * Self::MOVEMENT_SPEED;
        }
    }

    pub fn view_transform(&self) -> Matrix4<f32> {
        transforms::translate(Vector3::new(0.0, 0.0, -self.distance))
            * transforms::rotate_x(self.altitude)
            * transforms::rotate_y(self.azimuth)
            * transforms::translate(-self.center)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
