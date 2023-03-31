use super::{
    basic::{LinearTransformEntity, Translation},
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject},
};
use crate::{
    camera::Camera,
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, point_cloud::PointCloud},
    repositories::NameRepository,
};
use glow::HasContext;
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::{cell::RefCell, path::Path, rc::Rc};

pub struct Point<'gl> {
    position: Translation,
    point_cloud: PointCloud<'gl>,
    gl_program: GlProgram<'gl>,
    gl: &'gl glow::Context,
    size: f32,
    color: Color,
    name: ChangeableName,
}

impl<'gl> Point<'gl> {
    const DEFAULT_SIZE: f32 = 9.0;

    pub fn with_position(
        gl: &'gl glow::Context,
        position: Point3<f32>,
        name_repo: Rc<RefCell<dyn NameRepository>>,
    ) -> Self {
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
            name: ChangeableName::new("Point", name_repo),
        }
    }
}

impl<'gl> Entity for Point<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);
        self.position.control_ui(ui)
    }
}

impl<'gl> Drawable for Point<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, _draw_type: DrawType) {
        let model_transform = self.position.matrix();

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.gl_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        unsafe { self.gl.enable(glow::PROGRAM_POINT_SIZE) };
        self.gl_program.uniform_f32("point_size", self.size);
        self.gl_program
            .uniform_3_f32("point_color", self.color.r, self.color.g, self.color.b);

        self.point_cloud.draw();
    }
}

impl<'gl> SceneObject for Point<'gl> {
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
        if let Some(projected) = Point3::from_homogeneous(projected) {
            let is_at_point = (projected.x - point.x).abs() * resolution.width as f32 <= self.size
                && (projected.y - point.y).abs() * resolution.height as f32 <= self.size
                && projected.z > 0.0;

            let camera_distance =
                (self.position.translation - view_transform.column(3).xyz()).norm();

            (is_at_point, camera_distance)
        } else {
            (false, 0.0)
        }
    }

    fn location(&self) -> Option<Point3<f32>> {
        Some(self.position.translation.into())
    }

    fn model_transform(&self) -> Matrix4<f32> {
        self.position.matrix()
    }

    fn set_model_transform(&mut self, linear_transform: LinearTransformEntity) {
        self.position = linear_transform.translation;
    }

    fn is_single_point(&self) -> bool {
        true
    }
}

impl<'gl> NamedEntity for Point<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui)
    }
}
