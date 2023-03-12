use super::drawable::Drawable;
use crate::utils;
use glow::HasContext;
use nalgebra::{Matrix4, Point3};

pub struct LineMesh {
    points: Vec<Point3<f32>>,
    indices: Vec<u32>,
    model_transform: Matrix4<f32>,
    vertex_buffer: u32,
    vertex_array: u32,
}

impl LineMesh {
    pub fn new(gl: &glow::Context, points: Vec<Point3<f32>>, indices: Vec<u32>) -> LineMesh {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();
        let vertex_array = unsafe { gl.create_vertex_array() }.unwrap();

        unsafe {
            let raw_points = utils::slice_as_raw(&points);
            let raw_indices = utils::slice_as_raw(&indices);

            gl.bind_vertex_array(Some(vertex_array));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, raw_indices, glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                3 * std::mem::size_of::<f32>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.bind_vertex_array(None);
        }

        LineMesh {
            points,
            indices,
            model_transform: Matrix4::identity(),
            vertex_buffer,
            vertex_array,
        }
    }
}

impl Drop for LineMesh {
    fn drop(&mut self) {
        unsafe {}
    }
}

impl Drawable for LineMesh {
    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_elements(
                glow::LINES,
                self.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
            gl.bind_vertex_array(None);
        }
    }
}
