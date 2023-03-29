use crate::{math::affine::transforms, mouse::MouseState, window::Window};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use nalgebra::{Matrix4, Point2, Point3, Point4, Vector3, Vector4};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub azimuth: f32,
    pub altitude: f32,
    pub distance: f32,
    pub center: Point3<f32>,
    pub window_size: PhysicalSize<u32>,
}

impl Camera {
    const ROTATION_SPEED: f32 = 0.05;
    const MOVEMENT_SPEED: f32 = 0.01;

    pub fn new() -> Camera {
        Camera {
            azimuth: -std::f32::consts::FRAC_PI_4,
            altitude: std::f32::consts::FRAC_PI_4,
            distance: 5.0,
            center: Point3::new(0.0, 0.0, 0.0),
            window_size: PhysicalSize::new(0, 0),
        }
    }

    pub fn update_from_mouse(&mut self, mouse: &mut MouseState, window: &Window) -> bool {
        let mouse_delta = mouse.position_delta();
        let scroll_delta = mouse.scroll_delta();

        if (mouse_delta.x != 0.0 || mouse_delta.y != 0.0 || scroll_delta != 0.0)
            && !window.imgui_using_mouse()
        {
            self.update_angles(mouse, &mouse_delta);
            self.update_center(mouse, &mouse_delta);

            self.distance -= scroll_delta;

            if self.distance < 0.0 {
                self.distance = 0.0;
            }

            true
        } else {
            false
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

    pub fn position(&self) -> Point3<f32> {
        let homogeneous_position = self.inverse_view_transform() * Point4::new(0.0, 0.0, 0.0, 1.0);
        Point3::from_homogeneous(homogeneous_position.coords).unwrap()
    }

    pub fn view_transform(&self) -> Matrix4<f32> {
        transforms::translate(Vector3::new(0.0, 0.0, -self.distance))
            * transforms::rotate_x(self.altitude)
            * transforms::rotate_y(self.azimuth)
            * transforms::translate(-self.center.coords)
    }

    pub fn inverse_view_transform(&self) -> Matrix4<f32> {
        transforms::translate(self.center.coords)
            * transforms::rotate_y(-self.azimuth)
            * transforms::rotate_x(-self.altitude)
            * transforms::translate(Vector3::new(0.0, 0.0, self.distance))
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.window_size.width as f32 / self.window_size.height as f32
    }

    pub fn projection_transform(&self) -> Matrix4<f32> {
        transforms::projection(std::f32::consts::FRAC_PI_2, self.aspect_ratio(), 0.1, 100.0)
    }

    pub fn inverse_projection_transform(&self) -> Matrix4<f32> {
        transforms::inverse_projection(std::f32::consts::FRAC_PI_2, self.aspect_ratio(), 0.1, 100.0)
    }

    pub fn project_ray(&self, pixel: Point2<f32>) -> Vector3<f32> {
        let screen_point = Point4::new(pixel.x, pixel.y, -0.5, 1.0);

        Point3::from_homogeneous(
            self.inverse_view_transform()
                * transforms::inverse_projection(
                    std::f32::consts::FRAC_PI_2,
                    self.aspect_ratio(),
                    0.1,
                    100.0,
                )
                * Vector4::new(
                    screen_point.coords.x,
                    screen_point.coords.y,
                    screen_point.coords.z,
                    0.0,
                ),
        )
        .unwrap()
        .coords
        .normalize()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
