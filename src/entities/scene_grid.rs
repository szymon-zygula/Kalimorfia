use super::entity::{DrawType, Drawable};
use crate::{
    camera::Camera,
    math::affine::transforms,
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, mesh::LinesMesh},
};
use nalgebra::{Matrix4, Point3};
use std::path::Path;

pub struct SceneGrid<'gl> {
    mesh: LinesMesh<'gl>,
    gl_program: GlProgram<'gl>,
    scale: f32,
}

impl<'gl> SceneGrid<'gl> {
    pub fn new(gl: &'gl glow::Context, side_points: u32, scale: f32) -> SceneGrid<'gl> {
        let gl_program = GlProgram::with_shader_paths(
            gl,
            vec![
                (
                    Path::new("shaders/perspective_vertex.glsl"),
                    glow::VERTEX_SHADER,
                ),
                (
                    Path::new("shaders/simple_fragment.glsl"),
                    glow::FRAGMENT_SHADER,
                ),
            ],
        );

        SceneGrid {
            mesh: Self::grid_mesh(gl, side_points, side_points),
            gl_program,
            scale,
        }
    }

    fn grid_mesh(gl: &'gl glow::Context, points_x: u32, points_z: u32) -> LinesMesh {
        let mut points = Vec::new();
        let mut indices = Vec::new();

        for i in 0..points_x {
            for j in 0..points_z {
                let point = Point3::new(
                    2.0 / points_x as f32 * i as f32 - 1.0,
                    0.0,
                    2.0 / points_z as f32 * j as f32 - 1.0,
                );

                points.push(point);
                let idx = points.len() as u32 - 1;

                if j != 0 {
                    indices.push(idx);
                    indices.push(idx - 1);
                }

                if i != 0 {
                    indices.push(idx);
                    indices.push(idx - points_z);
                }
            }
        }

        LinesMesh::new(gl, points, indices)
    }
}

impl<'gl> Drawable for SceneGrid<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, _draw_type: DrawType) {
        let model_transform = transforms::scale(self.scale, 1.0, self.scale);

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.gl_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        self.mesh.draw();
    }
}
