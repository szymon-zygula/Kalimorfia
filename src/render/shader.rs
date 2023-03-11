use glow::{self, HasContext};

pub struct Shader {
    kind: u32,
    handle: u32,
}

impl Shader {
    pub fn from_file(gl: &glow::Context, shader_path: &std::path::Path, kind: u32) -> Shader {
        let shader_source =
            std::fs::read_to_string(shader_path).expect("Failed to load shader source code from");

        let handle = unsafe {
            let handle = gl.create_shader(kind).unwrap();
            gl.shader_source(handle, &shader_source);
            gl.compile_shader(handle);

            if !gl.get_shader_compile_status(handle) {
                panic!("Error compiling shader: {}", gl.get_shader_info_log(handle));
            }

            handle
        };

        Shader { kind, handle }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn kind(&self) -> u32 {
        self.kind
    }
}
