use super::shader::Shader;
use glow::{self, HasContext};

pub struct GlProgram {
    handle: u32,
    gl: std::rc::Rc<glow::Context>,
}

impl GlProgram {
    pub fn with_shader_paths(
        gl: std::rc::Rc<glow::Context>,
        shader_paths: Vec<(&std::path::Path, u32)>,
    ) -> GlProgram {
        let handle = unsafe { gl.create_program() }.unwrap();

        let shaders: Vec<Shader> = shader_paths
            .into_iter()
            .map(|(path, kind)| Shader::from_file(gl.as_ref(), path, kind))
            .collect();

        unsafe {
            for shader in &shaders {
                gl.attach_shader(handle, shader.handle());
            }

            gl.link_program(handle);

            if !gl.get_program_link_status(handle) {
                panic!(
                    "Error while linking shader: {}",
                    gl.get_program_info_log(handle)
                );
            }

            for shader in shaders {
                gl.detach_shader(handle, shader.handle());
            }
        }

        GlProgram { handle, gl }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn enable(&self) {
        unsafe {
            self.gl.use_program(Some(self.handle));
        }
    }
}

impl Drop for GlProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.handle);
        }
    }
}
