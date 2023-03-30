use super::entity::Entity;
use crate::math::affine::transforms;
use nalgebra::{Matrix4, Vector3};

pub struct Orientation {
    pub angle: f32,
    pub axis: Vector3<f32>,
}

impl Orientation {
    pub fn new() -> Orientation {
        Orientation {
            angle: 0.0,
            axis: Vector3::new(1.0, 0.0, 0.0),
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        transforms::rotate_axis(self.axis, self.angle)
    }

    pub fn inverse_matrix(&self) -> Matrix4<f32> {
        transforms::rotate_axis(self.axis, -self.angle)
    }

    pub fn reset(&mut self) {
        self.angle = 0.0;
        self.axis = Vector3::new(1.0, 0.0, 0.0);
    }
}

impl Entity for Orientation {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("orientation");
        let mut changed = false;

        ui.columns(2, "ancolumns", false);
        ui.text("Rotation angle");
        ui.next_column();
        changed |= imgui::AngleSlider::new("##angle")
            .range_degrees(0.0, 360.0)
            .display_format("%.2f°")
            .build(ui, &mut self.angle);
        ui.next_column();
        ui.columns(1, "ancolumns", false);

        ui.columns(4, "axcolumns", false);
        ui.text("Rotation axis");
        ui.next_column();

        changed |= ui.slider("x", -1.0, 1.0, &mut self.axis.x);
        ui.next_column();

        changed |= ui.slider("y", -1.0, 1.0, &mut self.axis.y);
        ui.next_column();

        changed |= ui.slider("z", -1.0, 1.0, &mut self.axis.z);
        ui.next_column();

        ui.columns(1, "axcolumns", false);

        changed
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
    const RANGE: f32 = 50.0;

    pub fn new() -> Translation {
        Self::with(Vector3::zeros())
    }

    pub fn with(translation: Vector3<f32>) -> Translation {
        Translation { translation }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        transforms::translate(self.translation)
    }

    pub fn inverse_matrix(&self) -> Matrix4<f32> {
        transforms::translate(-self.translation)
    }

    pub fn reset(&mut self) {
        self.translation = Vector3::zeros();
    }
}

impl Entity for Translation {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("translation");
        let mut changed = false;
        ui.columns(4, "columns", false);

        ui.text("Translation");
        ui.next_column();

        changed |= ui.slider("x", -Self::RANGE, Self::RANGE, &mut self.translation.x);
        ui.next_column();

        changed |= ui.slider("y", -Self::RANGE, Self::RANGE, &mut self.translation.y);
        ui.next_column();

        changed |= ui.slider("z", -Self::RANGE, Self::RANGE, &mut self.translation.z);
        ui.next_column();

        ui.columns(1, "columns", false);

        changed
    }
}

impl Default for Translation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Scale {
    pub scale: Vector3<f32>,
}

impl Scale {
    pub fn new() -> Scale {
        Scale {
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        transforms::scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn inverse_matrix(&self) -> Matrix4<f32> {
        transforms::scale(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z)
    }

    pub fn reset(&mut self) {
        self.scale = Vector3::new(1.0, 1.0, 1.0);
    }
}

impl Entity for Scale {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("scale");
        let mut changed = false;
        ui.columns(4, "columns", false);

        ui.text("Scale");
        ui.next_column();

        changed |= ui.slider("x", 0.0, 10.0, &mut self.scale.x);
        ui.next_column();

        changed |= ui.slider("y", 0.0, 10.0, &mut self.scale.y);
        ui.next_column();

        changed |= ui.slider("z", 0.0, 10.0, &mut self.scale.z);
        ui.next_column();

        ui.columns(1, "columns", false);

        changed
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Shear {
    pub xy: f32,
    pub xz: f32,
    pub yz: f32,
}

impl Shear {
    pub fn new() -> Shear {
        Shear {
            xy: 0.0,
            xz: 0.0,
            yz: 0.0,
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        transforms::shear_xy_xz_yz(self.xy, self.xz, self.yz)
    }

    pub fn inverse_matrix(&self) -> Matrix4<f32> {
        transforms::inverse_shear_xy_xz_yz(self.xy, self.xz, self.yz)
    }

    pub fn reset(&mut self) {
        self.xy = 0.0;
        self.xz = 0.0;
        self.yz = 0.0;
    }
}

impl Entity for Shear {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("scale");
        let mut changed = false;
        ui.columns(4, "columns", false);

        ui.text("Shear");
        ui.next_column();

        changed |= ui.slider("xy", -10.0, 10.0, &mut self.xy);
        ui.next_column();

        changed |= ui.slider("xz", -10.0, 10.0, &mut self.xz);
        ui.next_column();

        changed |= ui.slider("yz", -10.0, 10.0, &mut self.yz);
        ui.next_column();

        ui.columns(1, "columns", false);

        changed
    }
}

impl Default for Shear {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LinearTransformEntity {
    pub translation: Translation,
    pub orientation: Orientation,
    pub scale: Scale,
    pub shear: Shear,
}

impl LinearTransformEntity {
    pub fn new() -> Self {
        Self {
            translation: Translation::new(),
            orientation: Orientation::new(),
            scale: Scale::new(),
            shear: Shear::new(),
        }
    }

    pub fn reset(&mut self) {
        self.translation.reset();
        self.orientation.reset();
        self.scale.reset();
        self.shear.reset();
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.translation.matrix()
            * self.orientation.matrix()
            * self.shear.matrix()
            * self.scale.matrix()
    }

    pub fn inverse_matrix(&self) -> Matrix4<f32> {
        self.scale.inverse_matrix()
            * self.shear.inverse_matrix()
            * self.orientation.inverse_matrix()
            * self.translation.inverse_matrix()
    }
}

impl Entity for LinearTransformEntity {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let mut changed = false;

        ui.text("Transformations");
        changed |= self.translation.control_ui(ui);
        changed |= self.orientation.control_ui(ui);
        changed |= self.scale.control_ui(ui);
        changed |= self.shear.control_ui(ui);

        changed
    }
}

impl Default for LinearTransformEntity {
    fn default() -> Self {
        Self::new()
    }
}
