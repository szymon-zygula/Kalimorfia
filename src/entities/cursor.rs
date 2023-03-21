use super::{
    basic::Translation,
    entity::{Entity, SceneObject},
};
use crate::{
    math::affine::transforms,
    primitives::vertex::ColoredVertex,
    render::{drawable::Drawable, gl_program::GlProgram, mesh::ColoredLineMesh},
};
use nalgebra::{Matrix4, Point3};
use std::path::Path;

pub struct Cursor<'gl> {
    position: Translation,
    mesh: ColoredLineMesh<'gl>,
    gl_program: GlProgram<'gl>,
    scale: f32,
}

impl<'gl> Cursor<'gl> {
    pub fn new(gl: &glow::Context, scale: f32) -> Cursor {
        let mut mesh = ColoredLineMesh::new(
            gl,
            vec![
                ColoredVertex::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                ColoredVertex::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                ColoredVertex::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
                ColoredVertex::new(0.0, 1.0, 0.0, 0.0, 1.0, 0.0),
                ColoredVertex::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
                ColoredVertex::new(0.0, 0.0, 1.0, 0.0, 0.0, 1.0),
            ],
            vec![0, 1, 2, 3, 4, 5],
        );

        mesh.as_line_mesh_mut().thickness(3.0);

        let gl_program = GlProgram::with_shader_paths(
            gl,
            vec![
                (
                    Path::new("shaders/perspective_vertex_colored.glsl"),
                    glow::VERTEX_SHADER,
                ),
                (
                    Path::new("shaders/fragment_colored.glsl"),
                    glow::FRAGMENT_SHADER,
                ),
            ],
        );

        Cursor {
            position: Translation::new(),
            mesh,
            gl_program,
            scale,
        }
    }

    pub fn position(&self) -> Point3<f32> {
        self.position.translation.into()
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position.translation = position.coords;
    }
}

impl<'gl> Entity for Cursor<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        self.position.control_ui(ui);
    }
}

impl<'gl> SceneObject for Cursor<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let model_transform = self.position.as_matrix() * transforms::uniform_scale(self.scale);

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", model_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("projection_transform", projection_transform.as_slice());
        self.mesh.draw();
    }

    fn location(&self) -> Point3<f32> {
        self.position.translation.into()
    }
}
