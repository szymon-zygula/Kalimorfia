use super::gl_drawable::GlDrawable;
use crate::{render::opengl, utils};
use glow::HasContext;
use nalgebra::Point3;

pub struct PointCloud<'gl> {
    vertex_buffer: u32,
    vertex_array: u32,
    point_count: usize,
    gl: &'gl glow::Context,
}

impl<'gl> PointCloud<'gl> {
    pub fn new(gl: &'gl glow::Context, vertices: Vec<Point3<f32>>) -> PointCloud<'gl> {
        let mut mesh = Self::new_uninit(gl, vertices.len());

        mesh.vertex_array = opengl::init_vao(gl, || {
            mesh.update_points(vertices);

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

    fn new_uninit(gl: &'gl glow::Context, point_count: usize) -> PointCloud {
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        PointCloud {
            point_count,
            vertex_buffer,
            vertex_array: 0,
            gl,
        }
    }

    pub fn update_points(&mut self, points: Vec<Point3<f32>>) {
        let raw_points = utils::slice_as_raw(&points);

        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_points, glow::STATIC_DRAW);
        }
    }
}

impl<'gl> Drop for PointCloud<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}

impl<'gl> GlDrawable for PointCloud<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            self.gl
                .draw_arrays(glow::POINTS, 0, self.point_count as i32);
        });
    }
}
