use nalgebra::Matrix4;

pub trait Entity {
    fn control_ui(&mut self, ui: &imgui::Ui);
}

pub trait SceneObject {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>);
}
