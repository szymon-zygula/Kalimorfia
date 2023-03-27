pub(crate) use super::entity::ReferentialSceneEntity;
use nalgebra::Matrix4;
use std::{
    cell::{Ref, RefCell},
    collections::{BTreeMap, HashMap, HashSet},
};

#[derive(Default)]
pub struct EntityManager<'gl> {
    id_counter: usize,
    entities: BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    subscriptions: HashMap<usize, HashSet<usize>>,
}

impl<'gl> EntityManager<'gl> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn control_referential_ui(&mut self, controller_id: usize, ui: &imgui::Ui) {
        let result = self.entities[&controller_id]
            .borrow_mut()
            .control_referential_ui(ui, controller_id, &self.entities, &mut self.subscriptions);

        for (id, entity) in self.entities.iter().filter(|(&id, _)| id != controller_id) {
            entity.borrow_mut().notify_about_modification(
                &result
                    .intersection(&self.subscriptions[id])
                    .copied()
                    .collect(),
                &self.entities,
            );
        }
    }

    pub fn draw_referential(
        &self,
        id: usize,
        projection_transform: &Matrix4<f32>,
        view_transform: &Matrix4<f32>,
    ) {
        self.entities[&id].borrow().draw_referential(
            &self.entities,
            projection_transform,
            view_transform,
        );
    }

    pub fn remove_entity(&mut self, removed_id: usize) {
        self.entities.remove(&removed_id);

        for entity in self.entities.values() {
            entity
                .borrow_mut()
                .notify_about_deletion(&HashSet::from([removed_id]), &self.entities);
        }
    }

    pub fn add_entity(&mut self, entity: Box<dyn ReferentialSceneEntity<'gl> + 'gl>) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        self.entities.insert(id, RefCell::new(entity));
        self.subscriptions.insert(id, HashSet::new());
        id
    }

    pub fn get_entity_mut(&mut self, id: usize) -> &dyn ReferentialSceneEntity<'gl> {
        self.entities.get_mut(&id).unwrap().get_mut().as_mut()
    }

    pub fn get_entity(&self, id: usize) -> Ref<Box<dyn ReferentialSceneEntity<'gl> + 'gl>> {
        self.entities[&id].borrow()
    }

    pub fn entities_mut(
        &mut self,
    ) -> &mut BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>> {
        &mut self.entities
    }

    pub fn entities(
        &self,
    ) -> &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>> {
        &self.entities
    }

    pub fn subscribe(&mut self, subscriber: usize, subscribee: usize) {
        if self
            .subscriptions
            .get_mut(&subscriber)
            .unwrap()
            .insert(subscribee)
        {
            self.entities[&subscriber]
                .borrow_mut()
                .subscribe(subscribee, &self.entities);
        }
    }

    pub fn unsubscribe(&mut self, subscriber: usize, subscribee: usize) {
        if self
            .subscriptions
            .get_mut(&subscriber)
            .unwrap()
            .remove(&subscribee)
        {
            self.entities[&subscriber]
                .borrow_mut()
                .unsubscribe(subscribee, &self.entities);
        }
    }
}
