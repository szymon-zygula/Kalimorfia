use super::gl_program::GlProgram;
use std::collections::HashMap;

pub struct ShaderManager<'gl> {
    programs: HashMap<&'static str, GlProgram<'gl>>,
}

impl<'gl> ShaderManager<'gl> {
    pub fn new(programs: Vec<(&'static str, GlProgram<'gl>)>) -> ShaderManager<'gl> {
        Self {
            programs: programs.into_iter().collect(),
        }
    }

    pub fn program(&self, name: &str) -> &GlProgram<'gl> {
        &self.programs[name]
    }
}
