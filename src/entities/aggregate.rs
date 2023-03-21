use crate::{
    entities::{
        basic::{Orientation, Scale, Translation},
        cursor::Cursor,
        entity::{Entity, SceneEntity, SceneObject},
    },
    math::affine::transforms,
};
use nalgebra::{Matrix4, Point3, Vector3};
use std::collections::HashMap;

pub struct Aggregate<'gl> {
    cursor: Cursor<'gl>,
    rotation: Orientation,
    translation: Translation,
    scale: Scale,
    entities: HashMap<usize, Box<dyn SceneEntity + 'gl>>,
}

impl<'gl> Aggregate<'gl> {
    const CURSOR_SCALE: f32 = 1.0;
    pub fn new(gl: &'gl glow::Context) -> Aggregate<'gl> {
        Aggregate {
            cursor: Cursor::new(gl, Self::CURSOR_SCALE),
            rotation: Orientation::new(),
            translation: Translation::new(),
            scale: Scale::new(),
            entities: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, id: usize, object: Box<dyn SceneEntity + 'gl>) {
        self.entities.insert(id, object);
        self.reset_transform();
        self.cursor.set_position(self.location());
    }

    pub fn take_object(&mut self, id: usize) -> Box<dyn SceneEntity + 'gl> {
        let removed = self.entities.remove(&id).unwrap();
        self.reset_transform();
        self.cursor.set_position(self.location());
        removed
    }

    pub fn get_entity(&self, id: usize) -> &dyn SceneEntity {
        self.entities[&id].as_ref()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn only_one(&self) -> (usize, &dyn SceneEntity) {
        let (&id, boxed) = self.entities.iter().next().unwrap();
        (id, boxed.as_ref())
    }

    fn reset_transform(&mut self) {
        self.translation.translation = Vector3::zeros();
        self.rotation.reset();
        self.scale.reset();
    }
}

impl<'gl> SceneObject for Aggregate<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        match self.entities.len() {
            0 => {}
            1 => {
                self.cursor.draw(
                    projection_transform,
                    &(view_transform
                        * self.entities.values().next().unwrap().model_transform()
                        * transforms::translate(-self.cursor.location().coords)),
                );
            }
            _ => {
                self.cursor.draw(
                    projection_transform,
                    &(view_transform * self.model_transform()),
                );
            }
        }

        for entity in self.entities.values() {
            entity.draw(
                projection_transform,
                &(view_transform * self.model_transform()),
            );
        }
    }

    fn location(&self) -> Point3<f32> {
        if self.entities.is_empty() {
            return Point3::origin();
        }

        (Iterator::sum::<Vector3<f32>>(self.entities.values().map(|x| x.location().coords))
            / self.entities.len() as f32)
            .into()
    }

    fn model_transform(&self) -> Matrix4<f32> {
        transforms::translate(self.location().coords)
            * self.translation.as_matrix()
            * self.rotation.as_matrix()
            * self.scale.as_matrix()
            * transforms::translate(-self.location().coords)
    }
}

impl<'gl> Entity for Aggregate<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        match self.entities.len() {
            0 => {}
            1 => self.entities.values_mut().next().unwrap().control_ui(ui),
            n => {
                ui.text(format!("Control of {} entities", n));
                self.rotation.control_ui(ui);
                self.translation.control_ui(ui);
                self.scale.control_ui(ui);
            }
        }

        self.cursor.set_position(self.location());
    }
}
