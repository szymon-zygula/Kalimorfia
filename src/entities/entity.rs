use super::basic::LinearTransformEntity;
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

    fn location(&self) -> Point3<f32>;

    fn model_transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn set_model_transform(&mut self, _linear_transform: LinearTransformEntity) {
        panic!("Entity not is not transformable with LinearTransformEntity");
    }
}

pub trait SceneEntity: Entity + SceneObject {}

impl<T: Entity + SceneObject> SceneEntity for T {}
