use super::drawable::Drawable;
use crate::utils;
use glow::HasContext;
use nalgebra::{Matrix4, Point3};

pub struct LineMesh {
    index_count: u32,
    model_transform: Matrix4<f32>,
    vertex_buffer: u32,
    element_buffer: u32,
    vertex_array: u32,
    gl: std::rc::Rc<glow::Context>,
}

impl LineMesh {
    pub fn new(
        gl: std::rc::Rc<glow::Context>,
        points: Vec<Point3<f32>>,
        indices: Vec<u32>,
    ) -> LineMesh {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();
        let vertex_array = unsafe { gl.create_vertex_array() }.unwrap();

        let mut mesh = LineMesh {
            index_count: indices.len() as u32,
            model_transform: Matrix4::identity(),
            vertex_buffer,
            element_buffer,
            vertex_array,
            gl,
        };

        unsafe {
            mesh.gl.bind_vertex_array(Some(mesh.vertex_array));

            mesh.update_vertices(points, indices);
            mesh.gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                3 * std::mem::size_of::<f32>() as i32,
                0,
            );
            mesh.gl.enable_vertex_attrib_array(0);

            mesh.gl.bind_vertex_array(None);
        }

        mesh
    }

    pub fn update_vertices(&mut self, points: Vec<Point3<f32>>, indices: Vec<u32>) {
        let raw_points = utils::slice_as_raw(&points);
        let raw_indices = utils::slice_as_raw(&indices);

        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.element_buffer));
            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                raw_indices,
                glow::STATIC_DRAW,
            );
        }

        self.index_count = indices.len() as u32;
    }

    pub fn model_transform(&self) -> Matrix4<f32> {
        self.model_transform
    }

    pub fn transform(&mut self, transform: Matrix4<f32>) {
        self.model_transform = transform * self.model_transform;
    }
}

impl Drop for LineMesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.element_buffer);
        }
    }
}

impl Drawable for LineMesh {
    fn draw(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl
                .draw_elements(glow::LINES, self.index_count as i32, glow::UNSIGNED_INT, 0);
            self.gl.bind_vertex_array(None);
        }
    }
}
