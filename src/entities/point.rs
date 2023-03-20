use super::{
    basic::Translation,
    entity::{Entity, SceneObject},
};
use crate::{
    primitives::color::Color,
    render::{drawable::Drawable, gl_program::GlProgram, point_cloud::PointCloud},
};
use glow::HasContext;
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::path::Path;

pub struct Point<'gl> {
    position: Translation,
    point_cloud: PointCloud<'gl>,
    gl_program: GlProgram<'gl>,
    gl: &'gl glow::Context,
    size: f32,
    color: Color,
}

impl<'gl> Point<'gl> {
    const DEFAULT_SIZE: f32 = 6.0;

    pub fn with_position(gl: &'gl glow::Context, position: Point3<f32>) -> Point {
        let gl_program = GlProgram::with_shader_paths(
            gl,
            vec![
                (
                    Path::new("shaders/point_cloud_vertex.glsl"),
                    glow::VERTEX_SHADER,
                ),
                (
                    Path::new("shaders/fragment_colored.glsl"),
                    glow::FRAGMENT_SHADER,
                ),
            ],
        );

        Point {
            color: Color::white(),
            size: Self::DEFAULT_SIZE,
            gl,
            position: Translation::with(position.coords),
            gl_program,
            point_cloud: PointCloud::new(gl, vec![Point3::new(0.0, 0.0, 0.0)]),
        }
    }
}

impl<'gl> Entity for Point<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        ui.text("Point control");
        self.position.control_ui(ui);
    }
}

impl<'gl> SceneObject for Point<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let model_transform = self.position.as_matrix();

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", model_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("projection_transform", projection_transform.as_slice());

        unsafe { self.gl.enable(glow::PROGRAM_POINT_SIZE) };
        self.gl_program.uniform_f32("point_size", self.size);
        self.gl_program
            .uniform_3_f32("point_color", self.color.r, self.color.g, self.color.b);

        self.point_cloud.draw();
    }

    fn ray_intersects(&self, from: Point3<f32>, ray: Vector3<f32>) -> bool {
        if from.coords == self.position.translation {
            return false;
        }

        let to_point = (self.position.translation - from.coords).normalize();
        to_point.dot(&ray).abs() <= 0.1 && (to_point + ray).norm() > 1.0
    }

    fn is_at_point(
        &self,
        point: Point2<f32>,
        projection_transform: &Matrix4<f32>,
        view_transform: &Matrix4<f32>,
        resolution: &glutin::dpi::PhysicalSize<u32>,
    ) -> (bool, f32) {
        let projected = projection_transform
            * view_transform
            * Point3::from(self.position.translation).to_homogeneous();
        let projected = Point3::from_homogeneous(projected).unwrap();

        let is_at_point = (projected.x - point.x).abs() * resolution.width as f32 <= self.size
            && (projected.y - point.y).abs() * resolution.height as f32 <= self.size
            && projected.z > 0.0;

        let camera_distance = (self.position.translation - view_transform.column(3).xyz()).norm();

        (is_at_point, camera_distance)
    }
}
