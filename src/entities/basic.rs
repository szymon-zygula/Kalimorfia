use super::entity::Entity;
use crate::math::affine::transforms;
use nalgebra::{Matrix4, Vector3};

pub struct Orientation {
    angle: f32,
    axis: Vector3<f32>,
}

impl Orientation {
    pub fn new() -> Orientation {
        Orientation {
            angle: 0.0,
            axis: Vector3::new(1.0, 0.0, 0.0),
        }
    }

    pub fn as_matrix(&self) -> Matrix4<f32> {
        transforms::rotate_axis(self.axis, self.angle)
    }
}

impl Entity for Orientation {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        let token = ui.push_id("orientation");

        ui.columns(2, "ancolumns", false);
        ui.text("Rotation angle");
        ui.next_column();
        imgui::AngleSlider::new("##angle")
            .range_degrees(0.0, 360.0)
            .display_format("%.2f°")
            .build(ui, &mut self.angle);
        ui.next_column();
        ui.columns(1, "ancolumns", false);

        ui.columns(4, "axcolumns", false);
        ui.text("Rotation axis");
        ui.next_column();

        ui.slider("x", -1.0, 1.0, &mut self.axis.x);
        ui.next_column();

        ui.slider("y", -1.0, 1.0, &mut self.axis.y);
        ui.next_column();

        ui.slider("z", -1.0, 1.0, &mut self.axis.z);
        ui.next_column();

        ui.columns(1, "axcolumns", false);

        token.end();
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Translation {
    pub translation: Vector3<f32>,
}

impl Translation {
    pub fn new() -> Translation {
        Translation {
            translation: Vector3::zeros(),
        }
    }

    pub fn as_matrix(&self) -> Matrix4<f32> {
        transforms::translate(self.translation)
    }
}

impl Entity for Translation {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        let token = ui.push_id("translation");
        ui.columns(4, "columns", false);

        ui.text("Translation");
        ui.next_column();

        ui.slider("x", -10.0, 10.0, &mut self.translation.x);
        ui.next_column();

        ui.slider("y", -10.0, 10.0, &mut self.translation.y);
        ui.next_column();

        ui.slider("z", -10.0, 10.0, &mut self.translation.z);
        ui.next_column();

        ui.columns(1, "columns", false);
        token.end();
    }
}

impl Default for Translation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Scale {
    scale: Vector3<f32>,
}

impl Scale {
    pub fn new() -> Scale {
        Scale {
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn as_matrix(&self) -> Matrix4<f32> {
        transforms::scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

impl Entity for Scale {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        let token = ui.push_id("scale");
        ui.columns(4, "columns", false);

        ui.text("Scale");
        ui.next_column();

        ui.slider("x", 0.0, 10.0, &mut self.scale.x);
        ui.next_column();

        ui.slider("y", 0.0, 10.0, &mut self.scale.y);
        ui.next_column();

        ui.slider("z", 0.0, 10.0, &mut self.scale.z);
        ui.next_column();

        ui.columns(1, "columns", false);
        token.end();
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::new()
    }
}
