use super::basic::LinearTransformEntity;
use crate::camera::Camera;
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};

pub trait Entity {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool;
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
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize>;

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
    }

    fn notify_about_deletion(
        &mut self,
        _deleted: &HashSet<usize>,
        _remaining: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
    }

    fn subscribe(
        &mut self,
        _subscribees: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
    }

    fn unsubscribe(
        &mut self,
        _subscribees: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
    }

    fn add_point(
        &mut self,
        _id: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> bool {
        false
    }
}

impl<'gl, T: Entity> ReferentialEntity<'gl> for T {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize> {
        if self.control_ui(ui) {
            HashSet::from([controller_id])
        } else {
            HashSet::new()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DrawType {
    Normal,
    Selected,
}

pub trait Drawable {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType);

    fn draw_normal(&self, camera: &Camera) {
        self.draw(camera, &Matrix4::identity(), DrawType::Normal);
    }
}

pub trait ReferentialDrawable<'gl> {
    fn draw_referential(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    );
}

impl<'gl, T: Drawable> ReferentialDrawable<'gl> for T {
    fn draw_referential(
        &self,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
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

    fn model_transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }

    fn set_model_transform(&mut self, _linear_transform: LinearTransformEntity) {
        panic!("Entity not is not transformable with LinearTransformEntity");
    }

    fn is_single_point(&self) -> bool {
        false
    }
}

pub trait NamedEntity {
    fn name(&self) -> String;
    fn name_control_ui(&mut self, ui: &imgui::Ui);
}

pub trait SceneEntity: Entity + SceneObject + Drawable + NamedEntity {}
impl<T: Entity + SceneObject + Drawable + NamedEntity> SceneEntity for T {}

pub trait ReferentialSceneEntity<'gl>:
    ReferentialEntity<'gl> + SceneObject + ReferentialDrawable<'gl> + NamedEntity
{
}

impl<'gl, T: ReferentialEntity<'gl> + SceneObject + ReferentialDrawable<'gl> + NamedEntity>
    ReferentialSceneEntity<'gl> for T
{
}
