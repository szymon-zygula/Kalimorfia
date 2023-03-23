use super::entity::Entity;
use nalgebra::{Point2, Vector2};

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

    pub fn set_ndc_coords(&mut self, coordinates: Point2<f32>) {
        let coordinates = (coordinates + Vector2::new(1.0, 1.0)) / 2.0;
        self.coordinates.x = (coordinates.x * self.resolution.width as f32).round() as u32;
        self.coordinates.y = ((1.0 - coordinates.y) * self.resolution.height as f32).round() as u32;

        self.coordinates.x = std::cmp::min(
            std::cmp::max(0, self.coordinates.x),
            self.resolution.width - 1,
        );
        self.coordinates.y = std::cmp::min(
            std::cmp::max(0, self.coordinates.y),
            self.resolution.height - 1,
        );
    }

    pub fn get_ndc_coords(&self) -> Point2<f32> {
        2.0 * Point2::new(
            self.coordinates.x as f32 / self.resolution.width as f32 - 0.5,
            0.5 - self.coordinates.y as f32 / self.resolution.height as f32,
        )
    }

    pub fn set_coords(&mut self, coordinates: glutin::dpi::PhysicalPosition<u32>) {
        self.coordinates = coordinates;

        self.coordinates.x = std::cmp::min(
            std::cmp::max(0, self.coordinates.x),
            self.resolution.width - 1,
        );
        self.coordinates.y = std::cmp::min(
            std::cmp::max(0, self.coordinates.y),
            self.resolution.height - 1,
        );
    }

    pub fn set_resolution(&mut self, resolution: glutin::dpi::PhysicalSize<u32>) {
        self.resolution = resolution;
        self.set_coords(self.coordinates);
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

        self.set_coords(glutin::dpi::PhysicalPosition::new(x, y));

        changed
    }
}
