use nalgebra::{Matrix4, Point2, Point3, Vector3};

pub trait Entity {
    fn control_ui(&mut self, ui: &imgui::Ui);
}

pub trait SceneObject {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>);

    fn ray_intersects(&self, _from: Point3<f32>, _ray: Vector3<f32>) -> bool {
        false
    }

    fn is_at_point(
        &self,
        _point: Point2<f32>,
        _projection_transform: &Matrix4<f32>,
        _view_transform: &Matrix4<f32>,
        _resolution: &glutin::dpi::PhysicalSize<u32>,
    ) -> (bool, f32) {
        (false, 0.0)
    }
}

pub trait SceneEntity: Entity + SceneObject {}

impl<T: Entity + SceneObject> SceneEntity for T {}
