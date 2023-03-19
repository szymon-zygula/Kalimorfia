use glow::HasContext;

pub fn init_vao<F: FnOnce()>(gl: &glow::Context, initializer: F) -> u32 {
    unsafe {
        let vertex_array = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vertex_array));
        initializer();
        gl.bind_vertex_array(None);

        vertex_array
    }
}

pub fn with_vao<F: FnOnce()>(gl: &glow::Context, vertex_array: u32, action: F) {
    unsafe {
        gl.bind_vertex_array(Some(vertex_array));
        action();
        gl.bind_vertex_array(None);
    }
}
