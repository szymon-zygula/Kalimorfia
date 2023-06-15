use super::gl_drawable::GlDrawable;
use crate::{primitives::vertex::ColoredVertex, render::opengl, utils};
use glow::HasContext;
use nalgebra::{Point3, Vector2};

pub struct LinesMesh<'gl> {
    index_count: u32,
    vertex_buffer: u32,
    element_buffer: u32,
    vertex_array: u32,
    gl: &'gl glow::Context,
    thickness: f32,
}

impl<'gl> LinesMesh<'gl> {
    pub fn empty(gl: &'gl glow::Context) -> Self {
        Self::new(gl, Vec::new(), Vec::new())
    }

    pub fn new(gl: &'gl glow::Context, vertices: Vec<Point3<f32>>, indices: Vec<u32>) -> Self {
        let mut mesh = Self::new_uninit(gl, indices.len() as u32);

        mesh.vertex_array = opengl::init_vao(gl, || {
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

    pub fn strip(gl: &'gl glow::Context, vertices: Vec<Point3<f32>>) -> Self {
        let mut indices = Vec::with_capacity(vertices.len() * 2);
        for i in 0..(vertices.len() as u32 - 1) {
            indices.push(i);
            indices.push(i + 1);
        }

        Self::new(gl, vertices, indices)
    }

    fn new_uninit(gl: &'gl glow::Context, index_count: u32) -> LinesMesh {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();

        LinesMesh {
            index_count,
            vertex_buffer,
            element_buffer,
            vertex_array: 0,
            thickness: 1.0,
            gl,
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

impl<'gl> Drop for LinesMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.element_buffer);
        }
    }
}

impl<'gl> GlDrawable for LinesMesh<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            self.gl.line_width(self.thickness);
            self.gl
                .draw_elements(glow::LINES, self.index_count as i32, glow::UNSIGNED_INT, 0);
            self.gl.line_width(1.0);
        });
    }
}

pub struct ColoredLineMesh<'gl> {
    line_mesh: LinesMesh<'gl>,
}

impl<'gl> ColoredLineMesh<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        vertices: Vec<ColoredVertex>,
        indices: Vec<u32>,
    ) -> ColoredLineMesh<'gl> {
        let mut mesh = LinesMesh::new_uninit(gl, indices.len() as u32);

        mesh.vertex_array = opengl::init_vao(gl, || {
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

        ColoredLineMesh { line_mesh: mesh }
    }

    pub fn as_line_mesh_mut(&mut self) -> &mut LinesMesh<'gl> {
        &mut self.line_mesh
    }

    pub fn as_line_mesh(&self) -> &LinesMesh<'gl> {
        &self.line_mesh
    }
}

impl<'gl> GlDrawable for ColoredLineMesh<'gl> {
    fn draw(&self) {
        self.line_mesh.draw();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SurfaceVertex {
    pub point: Point3<f32>,
    pub uv: Vector2<f32>,
}

pub struct TorusMesh<'gl> {
    index_count: u32,
    vertex_buffer: u32,
    element_buffer: u32,
    vertex_array: u32,
    gl: &'gl glow::Context,
    thickness: f32,
}

impl<'gl> TorusMesh<'gl> {
    pub fn empty(gl: &'gl glow::Context) -> Self {
        Self::new(gl, Vec::new(), Vec::new())
    }

    pub fn new(gl: &'gl glow::Context, vertices: Vec<SurfaceVertex>, indices: Vec<u32>) -> Self {
        let mut mesh = Self::new_uninit(gl, indices.len() as u32);

        mesh.vertex_array = opengl::init_vao(gl, || {
            mesh.update_vertices(vertices, indices);

            unsafe {
                mesh.gl.vertex_attrib_pointer_f32(
                    0,
                    3,
                    glow::FLOAT,
                    false,
                    std::mem::size_of::<SurfaceVertex>() as i32,
                    0,
                );
                mesh.gl.enable_vertex_attrib_array(0);

                mesh.gl.vertex_attrib_pointer_f32(
                    1,
                    2,
                    glow::FLOAT,
                    false,
                    std::mem::size_of::<SurfaceVertex>() as i32,
                    std::mem::size_of::<Point3<f32>>() as i32,
                );
                mesh.gl.enable_vertex_attrib_array(1);
            }
        });

        mesh
    }

    fn new_uninit(gl: &'gl glow::Context, index_count: u32) -> Self {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();

        Self {
            index_count,
            vertex_buffer,
            element_buffer,
            vertex_array: 0,
            thickness: 1.0,
            gl,
        }
    }

    pub fn update_vertices(&mut self, points: Vec<SurfaceVertex>, indices: Vec<u32>) {
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

impl<'gl> Drop for TorusMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.element_buffer);
        }
    }
}

impl<'gl> GlDrawable for TorusMesh<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            self.gl.line_width(self.thickness);
            self.gl
                .draw_elements(glow::LINES, self.index_count as i32, glow::UNSIGNED_INT, 0);
            self.gl.line_width(1.0);
        });
    }
}
