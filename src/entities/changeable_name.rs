use super::entity::NamedEntity;
use crate::repositories::NameRepository;
use std::{cell::RefCell, rc::Rc};

pub struct ChangeableName {
    name_repo: Rc<RefCell<dyn NameRepository>>,
    name: String,
    rename: String,
}

impl ChangeableName {
    pub fn new(name: &str, name_repo: Rc<RefCell<dyn NameRepository>>) -> Self {
        let name = name_repo.borrow_mut().generate_name(name);
        Self {
            name_repo,
            rename: name.clone(),
            name,
        }
    }
}

impl NamedEntity for ChangeableName {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        ui.input_text("Name", &mut self.rename).build();

        if ui.button("Rename") {
            match self
                .name_repo
                .borrow_mut()
                .swap_name(&self.name, &self.rename)
            {
                Ok(new_name) => self.name = new_name,
                Err(_) => {
                    ui.open_popup("name_taken_popup");
                }
            }
        }

        ui.popup("name_taken_popup", || {
            ui.text("Name already taken");
        });
    }
}
