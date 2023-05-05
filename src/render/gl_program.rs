use super::shader::Shader;
use crate::primitives::color::Color;
use glow::{self, HasContext};

pub struct GlProgram<'gl> {
    handle: u32,
    gl: &'gl glow::Context,
}

macro_rules! fn_set_uniform {
    ($type:ty, $fn_name:ident) => {
        pub fn $fn_name(&self, name: &str, data: $type) {
            unsafe {
                let location = self.gl.get_uniform_location(self.handle, name).unwrap();
                self.gl.$fn_name(Some(&location), false, data);
            }
        }
    };
}

impl<'gl> GlProgram<'gl> {
    pub fn with_shaders(gl: &'gl glow::Context, shaders: &[&Shader]) -> GlProgram<'gl> {
        let handle = unsafe { gl.create_program() }.unwrap();

        unsafe {
            for shader in shaders {
                gl.attach_shader(handle, shader.handle());
            }

            gl.link_program(handle);

            if !gl.get_program_link_status(handle) {
                panic!("Error linking shader: {}", gl.get_program_info_log(handle));
            }

            for shader in shaders {
                gl.detach_shader(handle, shader.handle());
            }
        }

        GlProgram { handle, gl }
    }

    pub fn with_shader_paths(
        gl: &'gl glow::Context,
        shader_paths: Vec<(&std::path::Path, u32)>,
    ) -> GlProgram<'gl> {
        let shaders: Vec<Shader> = shader_paths
            .into_iter()
            .map(|(path, kind)| Shader::from_file(gl, path, kind))
            .collect();

        Self::with_shaders(gl, &shaders.iter().collect::<Vec<&Shader>>())
    }

    fn_set_uniform!(&[f32], uniform_matrix_2_f32_slice);
    fn_set_uniform!(&[f32], uniform_matrix_3_f32_slice);
    fn_set_uniform!(&[f32], uniform_matrix_4_f32_slice);

    pub fn uniform_f32(&self, name: &str, data: f32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_1_f32(Some(&location), data);
        }
    }

    pub fn uniform_u32(&self, name: &str, data: u32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_1_u32(Some(&location), data);
        }
    }

    pub fn uniform_3_f32(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            let location = self.gl.get_uniform_location(self.handle, name).unwrap();
            self.gl.uniform_3_f32(Some(&location), x, y, z);
        }
    }

    pub fn uniform_color(&self, name: &str, color: &Color) {
        self.uniform_3_f32(name, color.r, color.g, color.b);
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

impl<'gl> Drop for GlProgram<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.handle);
        }
    }
}
