use crate::entities::manager::EntityManager;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
};

pub struct Selector<S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)> {
    selectables: BTreeMap<usize, bool>,
    on_select: S,
    on_deselect: D,
    on_remove: R,
}

impl<S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)> Selector<S, D, R> {
    pub fn new(on_select: S, on_deselect: D, on_remove: R) -> Self {
        Self {
            selectables: BTreeMap::new(),
            on_select,
            on_deselect,
            on_remove,
        }
    }

    pub fn control_ui(
        &mut self,
        ui: &imgui::Ui,
        entity_manager: &RefCell<EntityManager>,
    ) -> (bool, HashSet<usize>) {
        ui.text("Object list");

        self.selectables.retain(|&id, selected| {
            let entity_manager = entity_manager.borrow();
            let entity = entity_manager.get_entity(id);
            let _token = ui.push_id(format!("entry_{}", entity.name()));

            ui.columns(2, "columns", false);
            let clicked = ui
                .selectable_config(entity.name())
                .selected(*selected)
                .build();

            drop(entity);
            drop(entity_manager);

            if clicked {
                *selected = !*selected;

                if *selected {
                    (self.on_select)(id);
                } else {
                    (self.on_deselect)(id);
                }
            }

            ui.next_column();
            let remove = ui.button_with_size("X", [18.0, 18.0]);
            if remove {
                (self.on_remove)(id);
            }

            ui.next_column();

            !remove
        });

        (false, HashSet::new())
    }

    pub fn add_selectable(&mut self, id: usize) {
        self.selectables.insert(id, false);
    }

    pub fn selected(&self) -> HashSet<usize> {
        self.selectables
            .iter()
            .filter(|(_, &selected)| selected)
            .map(|(id, _)| id)
            .copied()
            .collect()
    }

    pub fn unselected(&self) -> HashSet<usize> {
        self.selectables
            .iter()
            .filter(|(_, &selected)| !selected)
            .map(|(id, _)| id)
            .copied()
            .collect()
    }

    pub fn selectables(&self) -> HashSet<usize> {
        self.selectables.keys().copied().collect()
    }

    pub fn select(&mut self, id: usize) {
        (self.on_select)(id);
        *self.selectables.get_mut(&id).unwrap() = true;
    }

    pub fn deselect(&mut self, id: usize) {
        (self.on_deselect)(id);
        *self.selectables.get_mut(&id).unwrap() = false;
    }

    pub fn toggle(&mut self, id: usize) {
        if self.selectables[&id] {
            self.deselect(id);
        } else {
            self.select(id);
        }
    }
}
