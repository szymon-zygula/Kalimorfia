use crate::{
    math::affine::{screen::*, transforms},
    mouse::MouseState,
    window::Window,
};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use nalgebra::{Matrix4, Point2, Point3, Point4, Vector3, Vector4};

#[derive(Debug, Clone, PartialEq)]
pub struct Stereo {
    pub baseline: f32,
}

impl Default for Stereo {
    fn default() -> Self {
        Self::new()
    }
}

impl Stereo {
    pub fn new() -> Self {
        Self { baseline: 0.3 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub azimuth: f32,
    pub altitude: f32,
    pub log_distance: f32,
    pub center: Point3<f32>,
    pub resolution: PhysicalSize<u32>,
    pub near_plane: f32,
    pub far_plane: f32,
    pub screen_distance: f32,
    pub x_offset: f32,
    pub stereo: Option<Stereo>,
}

impl Camera {
    const ROTATION_SPEED: f32 = 0.05;
    const MOVEMENT_SPEED: f32 = 0.01;
    const SCROLL_SPEED: f32 = 0.2;

    pub fn new() -> Camera {
        Camera {
            azimuth: -std::f32::consts::FRAC_PI_4,
            altitude: std::f32::consts::FRAC_PI_4,
            log_distance: 2.0,
            center: Point3::new(0.0, 0.0, 0.0),
            resolution: PhysicalSize::new(0, 0),
            near_plane: 0.1,
            far_plane: 10000.0,
            x_offset: 0.0,
            screen_distance: 1.0,
            stereo: None,
        }
    }

    pub fn linear_distance(&self) -> f32 {
        self.log_distance.exp()
    }

    pub fn set_linear_distance(&mut self, linear_distance: f32) {
        self.log_distance = linear_distance.ln();
    }

    fn point_visible_with_tolerance(&self, point: &Point3<f32>, tolerance: f32) -> bool {
        Point3::from_homogeneous(
            self.projection_transform() * self.view_transform() * point.to_homogeneous(),
        )
        .map(|p| {
            p.x.abs() <= (1.0 + tolerance)
                && p.y.abs() <= (1.0 + tolerance)
                && p.z >= self.near_plane
                && p.z <= self.far_plane
        })
        .unwrap_or(false)
    }

    pub fn point_visible(&self, point: &Point3<f32>) -> bool {
        self.point_visible_with_tolerance(point, 0.0)
    }

    pub fn point_almost_visible(&self, point: &Point3<f32>) -> bool {
        self.point_visible_with_tolerance(point, 0.1)
    }

    pub fn update_from_mouse(&mut self, mouse: &mut MouseState, window: &Window) -> bool {
        let mouse_delta = mouse.position_delta();
        let scroll_delta = mouse.scroll_delta();

        if (mouse_delta.x != 0.0 || mouse_delta.y != 0.0 || scroll_delta != 0.0)
            && !window.imgui_using_mouse()
        {
            self.update_angles(mouse, &mouse_delta);
            self.update_center(mouse, &mouse_delta);

            self.log_distance -= Self::SCROLL_SPEED * scroll_delta;
            self.log_distance = self
                .log_distance
                .clamp(self.near_plane.ln(), self.far_plane.ln());

            true
        } else {
            false
        }
    }

    fn update_angles(&mut self, mouse: &MouseState, mouse_delta: &PhysicalPosition<f64>) {
        if mouse.is_middle_button_down() {
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
                * self.linear_distance()
                * Self::MOVEMENT_SPEED;
        }
    }

    pub fn position(&self) -> Point3<f32> {
        let homogeneous_position = self.inverse_view_transform() * Point4::new(0.0, 0.0, 0.0, 1.0);
        Point3::from_homogeneous(homogeneous_position.coords).unwrap()
    }

    pub fn view_transform(&self) -> Matrix4<f32> {
        transforms::translate(Vector3::new(0.0, 0.0, -self.linear_distance()))
            * transforms::rotate_x(self.altitude)
            * transforms::rotate_y(self.azimuth)
            * transforms::translate(-self.center.coords)
    }

    pub fn inverse_view_transform(&self) -> Matrix4<f32> {
        transforms::translate(self.center.coords)
            * transforms::rotate_y(-self.azimuth)
            * transforms::rotate_x(-self.altitude)
            * transforms::translate(Vector3::new(0.0, 0.0, self.linear_distance()))
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.resolution.width as f32 / self.resolution.height as f32
    }

    pub fn projection_transform(&self) -> Matrix4<f32> {
        transforms::unsymmetric_projection(
            self.aspect_ratio(),
            self.near_plane,
            self.far_plane,
            self.x_offset,
            self.screen_distance,
        )
    }

    pub fn inverse_projection_transform(&self) -> Matrix4<f32> {
        transforms::unsymmetric_projection_inverse(
            self.aspect_ratio(),
            self.near_plane,
            self.far_plane,
            self.x_offset,
            self.screen_distance,
        )
    }

    pub fn ray(&self, pixel: Point2<f32>) -> Vector3<f32> {
        let screen_point = Point4::new(pixel.x, pixel.y, -0.5, 1.0);

        Point3::from_homogeneous(
            self.inverse_view_transform()
                * self.inverse_projection_transform()
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

    pub fn world_to_ndc(&self, point: &Point3<f32>) -> Point3<f32> {
        Point3::from_homogeneous(
            self.projection_transform() * self.view_transform() * point.to_homogeneous(),
        )
        .unwrap_or(Point3::origin())
    }

    /// If `point` is behind the camera, the returned point will be in front of it (z' = |z| in
    /// camera space)
    pub fn ndc_to_world(&self, point: &Point3<f32>) -> Point3<f32> {
        let mut deprojected = self.inverse_projection_transform() * point.to_homogeneous();
        deprojected.z = -deprojected.z.abs();
        deprojected.w = deprojected.w.abs();
        Point3::from_homogeneous(self.inverse_view_transform() * deprojected)
            .unwrap_or(Point3::origin())
    }

    pub fn move_world_to_ndc(&self, old_world: &Point3<f32>, ndc: &Point2<f32>) -> Point3<f32> {
        let ndc_from_world = self.world_to_ndc(old_world);
        self.ndc_to_world(&Point3::new(ndc.x, ndc.y, ndc_from_world.z))
    }

    pub fn screen_to_ndc(&self, position: &PhysicalPosition<u32>) -> Point2<f32> {
        screen_to_ndc(&self.resolution, position)
    }

    pub fn ndc_to_screen(&self, position: &Point2<f32>) -> PhysicalPosition<u32> {
        ndc_to_screen(&self.resolution, position)
    }

    pub fn stereo_cameras(&self) -> Option<(Camera, Camera)> {
        self.stereo.as_ref().map(|stereo| {
            let inverse_view = self.inverse_view_transform();
            let view = self.view_transform();
            let shift = Vector4::new(stereo.baseline / 2.0, 0.0, 0.0, 0.0);
            let mut left = self.clone();
            left.x_offset = -stereo.baseline / 2.0;
            left.stereo = None;
            left.center = Point3::from_homogeneous(
                inverse_view * (view * self.center.to_homogeneous() + shift),
            )
            .unwrap();

            let mut right = self.clone();
            right.x_offset = stereo.baseline / 2.0;
            right.stereo = None;
            right.center = Point3::from_homogeneous(
                inverse_view * (view * self.center.to_homogeneous() - shift),
            )
            .unwrap();

            (left, right)
        })
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "focusPoint": {
                "x": self.center.x,
                "y": self.center.y,
                "z": self.center.z,
            },
            "distance": self.linear_distance(),
            "rotation": {
                "x": self.altitude,
                "y": self.azimuth
            }
        })
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
