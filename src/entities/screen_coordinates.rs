use super::entity::Entity;
use crate::math::affine::screen::*;
use nalgebra::Point2;

pub struct ScreenCoordinates {
    coordinates: glutin::dpi::PhysicalPosition<u32>,
    resolution: glutin::dpi::PhysicalSize<u32>,
}

impl ScreenCoordinates {
    pub fn new(resolution: glutin::dpi::PhysicalSize<u32>) -> ScreenCoordinates {
        ScreenCoordinates {
            coordinates: glutin::dpi::PhysicalPosition::new(0, 0),
            resolution,
        }
    }

    pub fn set_ndc(&mut self, coordinates: Point2<f32>) {
        self.coordinates = ndc_to_screen(&self.resolution, &coordinates);
    }

    pub fn get_ndc(&self) -> Point2<f32> {
        screen_to_ndc(&self.resolution, &self.coordinates)
    }

    pub fn set_screen_position(&mut self, coordinates: glutin::dpi::PhysicalPosition<u32>) {
        self.coordinates = clamp_screen(&self.resolution, &coordinates);
    }

    pub fn set_resolution(&mut self, resolution: glutin::dpi::PhysicalSize<u32>) {
        self.resolution = resolution;
        self.set_screen_position(self.coordinates);
    }
}

impl Entity for ScreenCoordinates {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("Screen coordinates");
        let mut changed = false;
        ui.columns(3, "columns", false);

        ui.text("Screen coordinates");
        ui.next_column();

        let mut x = self.coordinates.x;
        changed |= ui.slider("x", 0, self.resolution.width - 1, &mut x);
        ui.next_column();

        let mut y = self.coordinates.y;
        changed |= ui.slider("y", 0, self.resolution.height - 1, &mut y);
        ui.next_column();

        ui.columns(1, "columns", false);

        self.set_screen_position(glutin::dpi::PhysicalPosition::new(x, y));

        changed
    }
}
