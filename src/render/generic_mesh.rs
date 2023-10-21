use super::{gl_drawable::GlDrawable, opengl};
use crate::utils;
use glow::HasContext;
use nalgebra::{Point3, Vector3};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Triangle(pub [u32; 3]);

pub trait Vertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context);
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SimpleVertex(pub Point3<f32>);

impl SimpleVertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Point3::new(x, y, z))
    }
}

impl Vertex for SimpleVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<SimpleVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ClassicVertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl ClassicVertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self { position, normal }
    }
}

impl Vertex for ClassicVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<ClassicVertex>() as i32,
                std::mem::size_of::<Point3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(1);
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct CNCBlockVertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub x: f32,
    pub y: f32,
}

impl CNCBlockVertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>, x: f32, y: f32) -> Self {
        Self {
            position,
            normal,
            x,
            y,
        }
    }
}

impl Vertex for CNCBlockVertex {
    fn set_vertex_attrib_pointers(gl: &glow::Context) {
        unsafe {
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<CNCBlockVertex>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                std::mem::size_of::<CNCBlockVertex>() as i32,
                std::mem::size_of::<Point3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(1);

            gl.vertex_attrib_pointer_f32(
                2,
                1,
                glow::FLOAT,
                false,
                std::mem::size_of::<CNCBlockVertex>() as i32,
                std::mem::size_of::<Point3<f32>>() as i32
                    + std::mem::size_of::<Vector3<f32>>() as i32,
            );
            gl.enable_vertex_attrib_array(2);

            gl.vertex_attrib_pointer_f32(
                3,
                1,
                glow::FLOAT,
                false,
                std::mem::size_of::<CNCBlockVertex>() as i32,
                std::mem::size_of::<Point3<f32>>() as i32
                    + std::mem::size_of::<Vector3<f32>>() as i32
                    + std::mem::size_of::<f32>() as i32,
            );
            gl.enable_vertex_attrib_array(3);
        }
    }
}

pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub triangles: Vec<Triangle>,
}

pub struct GlMesh<'gl> {
    vertex_buffer: u32,
    element_buffer: u32,
    element_count: u32,
    vertex_array: u32,
    gl: &'gl glow::Context,
}

impl<'gl> GlMesh<'gl> {
    pub fn new<V: Vertex>(gl: &'gl glow::Context, mesh: &Mesh<V>) -> Self {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let element_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(gl, || unsafe {
            let raw_points = utils::slice_as_raw(&mesh.vertices);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);

            let raw_elements = utils::slice_as_raw(&mesh.triangles);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, raw_elements, glow::STATIC_DRAW);

            V::set_vertex_attrib_pointers(gl);
        });

        Self {
            vertex_buffer,
            element_buffer,
            element_count: 3 * mesh.triangles.len() as u32,
            vertex_array,
            gl,
        }
    }
}

impl<'gl> GlDrawable for GlMesh<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            self.gl.draw_elements(
                glow::TRIANGLES,
                self.element_count as i32,
                glow::UNSIGNED_INT,
                0,
            );
        });
    }
}

impl<'gl> Drop for GlMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
            self.gl.delete_buffer(self.element_buffer);
        }
    }
}
