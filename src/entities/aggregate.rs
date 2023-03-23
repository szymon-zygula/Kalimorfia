use crate::{
    entities::{
        basic::LinearTransformEntity,
        cursor::Cursor,
        entity::{Entity, SceneEntity, SceneObject},
    },
    math::{
        affine::transforms,
        decompositions::{axis_angle::AxisAngleDecomposition, trss::TRSSDecomposition},
    },
};
use nalgebra::{Matrix4, Point3, Vector3};
use std::collections::HashMap;

pub struct Aggregate<'gl> {
    cursor: Cursor<'gl>,
    linear_transform: LinearTransformEntity,
    entities: HashMap<usize, Box<dyn SceneEntity + 'gl>>,
}

impl<'gl> Aggregate<'gl> {
    const CURSOR_SCALE: f32 = 1.0;
    pub fn new(gl: &'gl glow::Context) -> Aggregate<'gl> {
        Aggregate {
            cursor: Cursor::new(gl, Self::CURSOR_SCALE),
            linear_transform: LinearTransformEntity::new(),
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
        self.linear_transform.reset()
    }

    fn basic_transform(&self, id: usize) -> LinearTransformEntity {
        let composed_transform = transforms::translate(self.cursor.location().coords)
            * self.linear_transform.as_matrix()
            * transforms::translate(-self.cursor.location().coords)
            * self.entities[&id].model_transform();

        let decomposed_transform = TRSSDecomposition::decompose(composed_transform);
        let axis_angle = AxisAngleDecomposition::decompose(&decomposed_transform.rotation);
        let mut linear_transform = LinearTransformEntity::new();

        linear_transform.translation.translation = decomposed_transform.translation;

        linear_transform.orientation.angle = axis_angle.angle;
        linear_transform.orientation.axis = axis_angle.axis;

        linear_transform.shear.xy = decomposed_transform.shear.x;
        linear_transform.shear.xz = decomposed_transform.shear.y;
        linear_transform.shear.yz = decomposed_transform.shear.z;

        linear_transform.scale.scale = decomposed_transform.scale;

        linear_transform
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
            * self.linear_transform.as_matrix()
            * transforms::translate(-self.location().coords)
    }
}

impl<'gl> Entity for Aggregate<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let changed = match self.entities.len() {
            0 => false,
            1 => self.entities.values_mut().next().unwrap().control_ui(ui),
            n => {
                ui.text(format!("Control of {} entities", n));

                let changed = self.linear_transform.control_ui(ui);

                if ui.button("Apply") {
                    for id in Iterator::collect::<Vec<usize>>(self.entities.keys().copied()) {
                        let tra = self.basic_transform(id);
                        self.entities.get_mut(&id).unwrap().set_model_transform(tra);
                    }

                    self.reset_transform();
                    true
                } else {
                    changed
                }
            }
        };

        self.cursor.set_position(self.location());

        changed
    }
}
