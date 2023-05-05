use crate::{
    camera::Camera,
    math::{geometry::bezier::BezierSurface, utils::point_64_to_32},
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, opengl},
    utils,
};
use glow::HasContext;
use nalgebra::{Matrix4, Point3};

#[repr(C)]
struct BezierPatchInput {
    points: [[Point3<f32>; 4]; 4],
}

pub struct BezierSurfaceMesh<'gl> {
    gl: &'gl glow::Context,
    vertex_buffer: u32,
    vertex_array: u32,
}

impl<'gl> BezierSurfaceMesh<'gl> {
    pub fn new(gl: &'gl glow::Context, surface: BezierSurface) -> Self {
        let mut patch_vertices = Vec::new();

        for patch_u in 0..surface.u_patches() {
            for patch_v in 0..surface.v_patches() {
                patch_vertices.push(BezierPatchInput {
                    points: [
                        [
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 0, 0)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 0, 1)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 0, 2)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 0, 3)),
                        ],
                        [
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 1, 0)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 1, 1)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 1, 2)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 1, 3)),
                        ],
                        [
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 2, 0)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 2, 1)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 2, 2)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 2, 3)),
                        ],
                        [
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 3, 0)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 3, 1)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 3, 2)),
                            point_64_to_32(surface.patch_point(patch_u, patch_v, 3, 3)),
                        ],
                    ],
                })
            }
        }

        let (vertex_array, vertex_buffer) = Self::create_vao_vbo(gl, patch_vertices);
        Self {
            gl,
            vertex_array,
            vertex_buffer,
        }
    }

    fn create_vao_vbo(gl: &'gl glow::Context, input: Vec<BezierPatchInput>) -> (u32, u32) {
        let raw_input = utils::slice_as_raw(&input);
        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();

        let vertex_array = opengl::init_vao(gl, || unsafe {
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

    pub fn draw_with_program(
        &self,
        program: &GlProgram,
        camera: &Camera,
        premul: &Matrix4<f32>,
        color: &Color,
        u_subdivisions: u32,
        v_subdivisions: u32,
    ) {
        program.enable();
        program.uniform_matrix_4_f32_slice("model", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice("projection", camera.projection_transform().as_slice());
        program.uniform_color("wireframe_color", color);
        program.uniform_u32("u_subdivisions", u_subdivisions);
        program.uniform_u32("v_subdivisions", v_subdivisions);

        self.draw();
    }
}

impl<'gl> GlDrawable for BezierSurfaceMesh<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            // TODO
            // todo!("Surface drawing")
        });
    }
}

impl<'gl> Drop for BezierSurfaceMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}
