use crate::{
    camera::Camera,
    entities::{
        basic::LinearTransformEntity,
        cursor::Cursor,
        entity::{
            DrawType, Drawable, Entity, NamedEntity, ReferentialDrawable, ReferentialEntity,
            ReferentialSceneEntity, SceneObject,
        },
    },
    math::{
        affine::transforms,
        decompositions::{axis_angle::AxisAngleDecomposition, trss::TRSSDecomposition},
    },
    repositories::NameRepository,
};
use nalgebra::{Matrix4, Point3, Vector3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};

pub struct Aggregate<'gl> {
    cursor: Cursor<'gl>,
    linear_transform: LinearTransformEntity,
    entities: HashSet<usize>,
    name: String,
}

impl<'gl> Aggregate<'gl> {
    const CURSOR_SCALE: f32 = 1.0;
    pub fn new(gl: &'gl glow::Context, name_repo: &mut dyn NameRepository) -> Aggregate<'gl> {
        Aggregate {
            cursor: Cursor::new(gl, Self::CURSOR_SCALE),
            linear_transform: LinearTransformEntity::new(),
            entities: HashSet::new(),
            name: name_repo.generate_name("Entity selection"),
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn only_one(&self) -> usize {
        let id = self.entities.iter().next().unwrap();
        *id
    }

    fn reset_transform(&mut self) {
        self.linear_transform.reset()
    }

    fn update_cursor(
        &mut self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        if self.entities.is_empty() {
            self.cursor.set_position(None);
            return;
        }

        let mut sum = Vector3::zeros();
        let mut count = 0.0;

        for id in &self.entities {
            if let Some(location) = entities[id].borrow().location() {
                sum += location.coords;
                count += 1.0;
            }
        }

        if count == 0.0 {
            self.cursor.set_position(None);
        } else {
            sum /= count;
            self.cursor.set_position(Some(sum.into()));
        }
    }

    fn composed_transform(&self, transform: &Matrix4<f32>) -> LinearTransformEntity {
        let composed_transform = transforms::translate(self.cursor.location().unwrap().coords)
            * self.linear_transform.as_matrix()
            * transforms::translate(-self.cursor.location().unwrap().coords)
            * transform;

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
    fn location(&self) -> Option<Point3<f32>> {
        self.cursor.location()
    }

    fn model_transform(&self) -> Matrix4<f32> {
        if let Some(location) = self.location() {
            transforms::translate(location.coords)
                * self.linear_transform.as_matrix()
                * transforms::translate(-self.location().unwrap().coords)
        } else {
            Matrix4::identity()
        }
    }
}

impl<'gl> ReferentialDrawable<'gl> for Aggregate<'gl> {
    fn draw_referential(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        match self.entities.len() {
            0 => {}
            1 => {
                let only_id = self.entities.iter().next().unwrap();
                if let Some(ref location) = self.cursor.location() {
                    if entities[only_id].borrow().location().is_some() {
                        self.cursor.draw(
                            camera,
                            &(premul
                                * entities[only_id].borrow().model_transform()
                                * transforms::translate(-location.coords)),
                            draw_type,
                        );
                    }
                }
            }
            _ => {
                self.cursor.draw(camera, &self.model_transform(), draw_type);
            }
        }

        for id in &self.entities {
            if entities[id].borrow().location().is_some() {
                entities[id].borrow().draw_referential(
                    entities,
                    camera,
                    &(premul * self.model_transform()),
                    draw_type,
                );
            } else {
                entities[id]
                    .borrow()
                    .draw_referential(entities, camera, premul, draw_type);
            }
        }
    }
}

impl<'gl> ReferentialEntity<'gl> for Aggregate<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize> {
        let changes = match self.entities.len() {
            0 => HashSet::new(),
            1 => {
                let id = self.entities.iter().next().unwrap();
                entities[id]
                    .borrow_mut()
                    .control_referential_ui(ui, *id, entities, subscriptions)
            }
            n => {
                ui.text(format!("Control of {} entities", n));

                let changed = self.linear_transform.control_ui(ui);

                if ui.button("Apply") {
                    for id in &self.entities {
                        let transform =
                            self.composed_transform(&entities[id].borrow().model_transform());

                        if entities[id].borrow().location().is_some() {
                            entities[id].borrow_mut().set_model_transform(transform);
                        }
                    }

                    self.update_cursor(entities);
                    self.reset_transform();
                    let mut changes = self.entities.clone();
                    changes.insert(controller_id);
                    changes
                } else if changed {
                    HashSet::from([controller_id])
                } else {
                    HashSet::new()
                }
            }
        };

        changes
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.update_cursor(entities);
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        remaining: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.entities = self.entities.difference(deleted).copied().collect();
        self.update_cursor(remaining);
    }

    fn subscribe(
        &mut self,
        subscribee: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.entities.insert(subscribee);
        self.reset_transform();
        self.update_cursor(entities);
    }

    fn unsubscribe(
        &mut self,
        subscribee: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.entities.remove(&subscribee);
    }
}

impl<'gl> NamedEntity for Aggregate<'gl> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn name_control_ui(&mut self, _ui: &imgui::Ui) {}
}
