use crate::entities::manager::EntityManager;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
};

pub struct Selector<'a> {
    selectables: BTreeMap<usize, bool>,
    on_select: Box<dyn FnMut(usize) + 'a>,
    on_deselect: Box<dyn FnMut(usize) + 'a>,
    on_remove: Box<dyn FnMut(usize) -> Option<String> + 'a>,
    last_remove_info: Option<String>,
}

impl<'a> Selector<'a> {
    pub fn new(
        on_select: impl FnMut(usize) + 'a,
        on_deselect: impl FnMut(usize) + 'a,
        on_remove: impl FnMut(usize) -> Option<String> + 'a,
    ) -> Self {
        Self {
            selectables: BTreeMap::new(),
            on_select: Box::new(on_select),
            on_deselect: Box::new(on_deselect),
            on_remove: Box::new(on_remove),
            last_remove_info: None,
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
                .selectable_config(format!("[{}] {}", id, entity.name()))
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
            let mut remove = ui.button_with_size("X", [18.0, 18.0]);
            if remove {
                if let Some(rejection_info) = (self.on_remove)(id) {
                    ui.open_popup("removal_info");
                    self.last_remove_info = Some(rejection_info);
                    remove = false;
                }
            }

            ui.popup("removal_info", || {
                ui.text(self.last_remove_info.as_ref().unwrap());
            });

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

    pub fn only_selected(&self) -> Option<usize> {
        let selected = self.selected();
        if selected.len() == 1 {
            Some(*selected.iter().next().unwrap())
        } else {
            None
        }
    }

    pub fn unselected(&self) -> HashSet<usize> {
        self.selectables
            .iter()
            .filter(|(_, &selected)| !selected)
            .map(|(id, _)| id)
            .copied()
            .collect()
    }

    pub fn selectables(&self) -> &BTreeMap<usize, bool> {
        &self.selectables
    }

    pub fn select(&mut self, id: usize) {
        (self.on_select)(id);
        *self.selectables.get_mut(&id).unwrap() = true;
    }

    pub fn select_all(&mut self) {
        for (id, selected) in &mut self.selectables {
            if !*selected {
                *selected = true;
                (self.on_select)(*id);
            }
        }
    }

    pub fn deselect(&mut self, id: usize) {
        (self.on_deselect)(id);
        *self.selectables.get_mut(&id).unwrap() = false;
    }

    pub fn deselect_all(&mut self) {
        for (id, selected) in &mut self.selectables {
            if *selected {
                *selected = false;
                (self.on_deselect)(*id);
            }
        }
    }

    pub fn remove(&mut self, id: usize) {
        self.selectables.remove(&id);
    }

    pub fn toggle(&mut self, id: usize) -> bool {
        if self.selectables[&id] {
            self.deselect(id);
            false
        } else {
            self.select(id);
            true
        }
    }

    pub fn reset(&mut self) {
        self.selectables.clear();
        self.last_remove_info = None;
    }
}
