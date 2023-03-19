use glow::{self, HasContext};

pub struct Shader<'g> {
    kind: u32,
    handle: u32,
    gl: &'g glow::Context,
}

impl<'g> Shader<'g> {
    pub fn from_file(
        gl: &'g glow::Context,
        shader_path: &std::path::Path,
        kind: u32,
    ) -> Shader<'g> {
        let shader_source =
            std::fs::read_to_string(shader_path).expect("Failed to load shader source code from");

        let handle = unsafe {
            let handle = gl.create_shader(kind).unwrap();
            gl.shader_source(handle, &shader_source);
            gl.compile_shader(handle);

            if !gl.get_shader_compile_status(handle) {
                panic!(
                    "Error compiling shader ({}): {}",
                    shader_path.to_str().unwrap(),
                    gl.get_shader_info_log(handle)
                );
            }

            handle
        };

        Shader { kind, handle, gl }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn kind(&self) -> u32 {
        self.kind
    }
}

impl<'g> Drop for Shader<'g> {
    fn drop(&mut self) {
        unsafe { self.gl.delete_shader(self.handle) };
    }
}
