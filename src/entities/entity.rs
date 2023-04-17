use super::{basic::LinearTransformEntity, point::Point};
use crate::camera::Camera;
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};

pub trait Entity {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool;
}

#[derive(Clone, Default)]
pub struct ControlResult {
    pub modified: HashSet<usize>,
    pub notification_excluded: HashSet<usize>,
}

pub trait ReferentialEntity<'gl> {
    /// In contrast to a regular `Entity`, a `ReferentialEntity` can also control any other
    /// entities passed to its `control_ui`.
    /// Returns information about wheather the entity has been modified and a `Vec` of ids of
    /// entities that were modified.
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        id: usize,
        entities: &EntityCollection<'gl>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult;

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        _entities: &EntityCollection<'gl>,
    ) {
    }

    fn notify_about_deletion(
        &mut self,
        _deleted: &HashSet<usize>,
        _remaining: &EntityCollection<'gl>,
    ) {
    }

    fn subscribe(&mut self, _subscribees: usize, _entities: &EntityCollection<'gl>) {}

    fn unsubscribe(&mut self, _subscribees: usize, _entities: &EntityCollection<'gl>) {}

    fn add_point(&mut self, _id: usize, _entities: &EntityCollection<'gl>) -> bool {
        false
    }
}

impl<'gl, T: Entity> ReferentialEntity<'gl> for T {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        if self.control_ui(ui) {
            ControlResult {
                modified: HashSet::from([controller_id]),
                ..Default::default()
            }
        } else {
            ControlResult::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawType {
    Regular,
    Selected,
    Virtual,
    SelectedVirtual,
}

pub trait Drawable {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType);

    fn draw_regular(&self, camera: &Camera) {
        self.draw(camera, &Matrix4::identity(), DrawType::Regular);
    }
}

pub trait ReferentialDrawable<'gl> {
    fn draw_referential(
        &self,
        entities: &EntityCollection<'gl>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    );
}

impl<'gl, T: Drawable> ReferentialDrawable<'gl> for T {
    fn draw_referential(
        &self,
        _entities: &EntityCollection<'gl>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        self.draw(camera, premul, draw_type);
    }
}

pub trait SceneObject {
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

    fn location(&self) -> Option<Point3<f32>> {
        None
    }

    fn set_ndc(&mut self, _ndc: &Point2<f32>, _camera: &Camera) {}

    fn model_transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn set_model_transform(&mut self, _linear_transform: LinearTransformEntity) {
        panic!("Entity not is not transformable with LinearTransformEntity");
    }

    fn is_single_point(&self) -> bool {
        false
    }

    fn as_point(&self) -> Option<&Point> {
        None
    }
}

pub trait ReferentialSceneObject<'gl> {
    fn set_ndc(
        &mut self,
        _ndc: &Point2<f32>,
        _camera: &Camera,
        _entities: &EntityCollection<'gl>,
        _controller_id: usize,
    ) -> ControlResult {
        ControlResult::default()
    }

    fn is_at_point(
        &mut self,
        _point: Point2<f32>,
        _projection_transform: &Matrix4<f32>,
        _view_transform: &Matrix4<f32>,
        _resolution: &glutin::dpi::PhysicalSize<u32>,
        _entities: &EntityCollection<'gl>,
    ) -> (bool, f32) {
        (false, 0.0)
    }

    fn ray_intersects(&self, _from: Point3<f32>, _ray: Vector3<f32>) -> bool {
        false
    }

    fn location(&self) -> Option<Point3<f32>> {
        None
    }

    fn model_transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn set_model_transform(&mut self, _linear_transform: LinearTransformEntity) {
        panic!("Entity not is not transformable with LinearTransformEntity");
    }

    fn is_single_point(&self) -> bool {
        false
    }

    fn as_point(&self) -> Option<&Point> {
        None
    }
}

impl<'gl, T: SceneObject> ReferentialSceneObject<'gl> for T {
    fn set_ndc(
        &mut self,
        ndc: &Point2<f32>,
        camera: &Camera,
        _entities: &EntityCollection<'gl>,
        controller_id: usize,
    ) -> ControlResult {
        self.set_ndc(ndc, camera);
        ControlResult {
            modified: HashSet::from([controller_id]),
            ..Default::default()
        }
    }

    fn is_at_point(
        &mut self,
        point: Point2<f32>,
        projection_transform: &Matrix4<f32>,
        view_transform: &Matrix4<f32>,
        resolution: &glutin::dpi::PhysicalSize<u32>,
        _entities: &EntityCollection<'gl>,
    ) -> (bool, f32) {
        SceneObject::is_at_point(
            self,
            point,
            projection_transform,
            view_transform,
            resolution,
        )
    }

    fn ray_intersects(&self, from: Point3<f32>, ray: Vector3<f32>) -> bool {
        self.ray_intersects(from, ray)
    }

    fn location(&self) -> Option<Point3<f32>> {
        self.location()
    }

    fn model_transform(&self) -> Matrix4<f32> {
        self.model_transform()
    }

    fn set_model_transform(&mut self, linear_transform: LinearTransformEntity) {
        self.set_model_transform(linear_transform)
    }

    fn is_single_point(&self) -> bool {
        self.is_single_point()
    }

    fn as_point(&self) -> Option<&Point> {
        self.as_point()
    }
}

pub trait NamedEntity {
    fn name(&self) -> String;
    fn name_control_ui(&mut self, ui: &imgui::Ui);
}

pub trait SceneEntity: Entity + SceneObject + Drawable + NamedEntity {}
impl<T: Entity + SceneObject + Drawable + NamedEntity> SceneEntity for T {}

pub trait ReferentialSceneEntity<'gl>:
    ReferentialEntity<'gl> + ReferentialSceneObject<'gl> + ReferentialDrawable<'gl> + NamedEntity
{
}

impl<'gl, T> ReferentialSceneEntity<'gl> for T where
    T: ReferentialEntity<'gl>
        + ReferentialSceneObject<'gl>
        + ReferentialDrawable<'gl>
        + NamedEntity
{
}

pub type EntityCollection<'gl> =
    BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>;
