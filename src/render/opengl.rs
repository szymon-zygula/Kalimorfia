use glow::HasContext;
use nalgebra::Point3;

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

pub fn create_vao_vbo_points(gl: &glow::Context, raw_input: &[u8]) -> (u32, u32) {
    let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

    let vertex_array = init_vao(gl, || unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_input, glow::STATIC_DRAW);

        gl.vertex_attrib_pointer_f32(
            0,
            3,
            glow::FLOAT,
            false,
            std::mem::size_of::<Point3<f32>>() as i32,
            0,
        );
        gl.enable_vertex_attrib_array(0);
    });

    (vertex_array, vertex_buffer)
}
