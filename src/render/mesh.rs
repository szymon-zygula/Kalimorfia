use super::drawable::Drawable;
use crate::{primitives::vertex::ColoredVertex, utils};
use glow::HasContext;
use nalgebra::Point3;

pub struct LineMesh<'gl> {
    index_count: u32,
    vertex_buffer: u32,
    element_buffer: u32,
    vertex_array: u32,
    gl: &'gl glow::Context,
    thickness: f32,
}

impl<'gl> LineMesh<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        vertices: Vec<Point3<f32>>,
        indices: Vec<u32>,
    ) -> LineMesh<'gl> {
        let mut mesh = Self::new_uninit(gl, indices.len() as u32);

        mesh.init_vao(|mesh| {
            mesh.update_vertices(vertices, indices);

            unsafe {
                mesh.gl.vertex_attrib_pointer_f32(
                    0,
                    3,
                    glow::FLOAT,
                    false,
                    3 * std::mem::size_of::<f32>() as i32,
                    0,
                );
                mesh.gl.enable_vertex_attrib_array(0);
            }
        });

        mesh
    }

    fn new_uninit(gl: &'gl glow::Context, index_count: u32) -> LineMesh {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();
        let vertex_array = unsafe { gl.create_vertex_array() }.unwrap();

        LineMesh {
            index_count,
            vertex_buffer,
            element_buffer,
            vertex_array,
            thickness: 1.0,
            gl,
        }
    }

    fn init_vao<F: FnOnce(&mut LineMesh)>(&mut self, vao_initializer: F) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vertex_array));
            vao_initializer(self);
            self.gl.bind_vertex_array(None);
        }
    }

    pub fn update_vertices<T>(&mut self, points: Vec<T>, indices: Vec<u32>) {
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

    pub fn thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }
}

impl<'gl> Drop for LineMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.element_buffer);
        }
    }
}

impl<'gl> Drawable for LineMesh<'gl> {
    fn draw(&self) {
        unsafe {
            self.gl.line_width(self.thickness);
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl
                .draw_elements(glow::LINES, self.index_count as i32, glow::UNSIGNED_INT, 0);
            self.gl.bind_vertex_array(None);
            self.gl.line_width(1.0);
        }
    }
}

pub struct ColoredLineMesh<'gl> {
    line_mesh: LineMesh<'gl>,
}

impl<'gl> ColoredLineMesh<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        vertices: Vec<ColoredVertex>,
        indices: Vec<u32>,
    ) -> ColoredLineMesh<'gl> {
        let mut line_mesh = LineMesh::new_uninit(gl, indices.len() as u32);

        line_mesh.init_vao(|mesh| {
            mesh.update_vertices(vertices, indices);

            unsafe {
                mesh.gl.vertex_attrib_pointer_f32(
                    0,
                    3,
                    glow::FLOAT,
                    false,
                    6 * std::mem::size_of::<f32>() as i32,
                    0,
                );
                mesh.gl.enable_vertex_attrib_array(0);

                mesh.gl.vertex_attrib_pointer_f32(
                    1,
                    3,
                    glow::FLOAT,
                    false,
                    6 * std::mem::size_of::<f32>() as i32,
                    3 * std::mem::size_of::<f32>() as i32,
                );
                mesh.gl.enable_vertex_attrib_array(1);
            }
        });

        ColoredLineMesh { line_mesh }
    }

    pub fn as_line_mesh_mut(&mut self) -> &mut LineMesh<'gl> {
        &mut self.line_mesh
    }

    pub fn as_line_mesh(&self) -> &LineMesh<'gl> {
        &self.line_mesh
    }
}

impl<'gl> Drawable for ColoredLineMesh<'gl> {
    fn draw(&self) {
        self.line_mesh.draw();
    }
}
