use std::collections::{HashMap, HashSet};

pub enum NameError {
    NameTaken,
}

pub trait NameRepository {
    fn generate_name(&mut self, name: &str) -> String;
    fn take_name(&mut self, name: &str) -> Result<String, NameError>;
    fn return_name(&mut self, name: &str);

    fn swap_name(&mut self, old_name: &str, new_name: &str) -> Result<String, NameError> {
        self.return_name(old_name);
        self.take_name(new_name)
    }
}

#[derive(Default)]
pub struct UniqueNameRepository {
    name_counters: HashMap<String, usize>,
    names: HashSet<String>,
}

impl UniqueNameRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NameRepository for UniqueNameRepository {
    fn generate_name(&mut self, name: &str) -> String {
        let number = match self.name_counters.get_mut(name) {
            Some(count) => {
                *count += 1;
                *count
            }
            None => {
                self.name_counters.insert(String::from(name), 0);
                0
            }
        };

        let entity_name = format!("{} {}", name, number);
        if !self.names.insert(entity_name.clone()) {
            // Generate the next name if this one is taken already
            return self.generate_name(name);
        }

        entity_name
    }

    fn take_name(&mut self, name: &str) -> Result<String, NameError> {
        let name = String::from(name);
        if self.names.insert(name.clone()) {
            Ok(name)
        } else {
            Err(NameError::NameTaken)
        }
    }

    fn return_name(&mut self, name: &str) {
        self.names.remove(name);
    }
}

#[derive(Default)]
pub struct ExactNameRepository {}

impl ExactNameRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NameRepository for ExactNameRepository {
    fn generate_name(&mut self, name: &str) -> String {
        String::from(name)
    }

    fn take_name(&mut self, name: &str) -> Result<String, NameError> {
        Ok(String::from(name))
    }

    fn return_name(&mut self, _name: &str) {}
}
