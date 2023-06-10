use super::{
    basic::{LinearTransformEntity, Translation},
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject},
};
use crate::{
    camera::Camera,
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, point_cloud::PointCloud, shader_manager::ShaderManager},
    repositories::NameRepository,
};
use glow::HasContext;
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub struct Point<'gl> {
    position: Translation,
    point_cloud: PointCloud<'gl>,
    gl: &'gl glow::Context,
    size: f32,
    name: ChangeableName,
    shader_manager: Rc<ShaderManager<'gl>>,
    pub color: Color,
}

impl<'gl> Point<'gl> {
    const DEFAULT_SIZE: f32 = 9.0;

    pub fn with_position(
        gl: &'gl glow::Context,
        position: Point3<f32>,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Self {
        Point {
            shader_manager,
            size: Self::DEFAULT_SIZE,
            gl,
            position: Translation::with(position.coords),
            point_cloud: PointCloud::new(gl, vec![Point3::new(0.0, 0.0, 0.0)]),
            name: ChangeableName::new("Point", name_repo),
            color: Color::white(),
        }
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_position(&mut self, new_position: Vector3<f32>) {
        self.position.translation = new_position;
    }

    fn color_control(&mut self, ui: &imgui::Ui) {
        let mut color = [self.color.r, self.color.g, self.color.b];
        ui.color_edit3("Color", &mut color);
        self.color.r = color[0];
        self.color.g = color[1];
        self.color.b = color[2];
    }
}

impl<'gl> Entity for Point<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);
        self.color_control(ui);
        self.position.control_ui(ui)
    }
}

impl<'gl> Drawable for Point<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let model_transform = self.position.matrix();

        let program = self.shader_manager.program("point");
        program.enable();
        program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        unsafe { self.gl.enable(glow::PROGRAM_POINT_SIZE) };
        program.uniform_f32("point_size", self.size);

        let color = match draw_type {
            DrawType::Regular => self.color,
            _ => Color::for_draw_type(&draw_type),
        };

        program.uniform_color("point_color", &color);

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

    fn is_at_ndc(&self, point: Point2<f32>, camera: &Camera) -> Option<f32> {
        let projected = camera.projection_transform()
            * camera.view_transform()
            * Point3::from(self.position.translation).to_homogeneous();

        if let Some(projected) = Point3::from_homogeneous(projected) {
            let is_at_point = (projected.x - point.x).abs() * camera.resolution.width as f32
                <= self.size
                && (projected.y - point.y).abs() * camera.resolution.height as f32 <= self.size
                && projected.z < 0.0;

            if !is_at_point {
                return None;
            }

            let camera_distance = (self.position.translation - camera.position().coords).norm();

            Some(camera_distance)
        } else {
            None
        }
    }

    fn location(&self) -> Option<Point3<f32>> {
        Some(self.position.translation.into())
    }

    fn set_ndc<'a>(&mut self, ndc: &Point2<f32>, camera: &Camera) {
        self.position.set_ndc(ndc, camera);
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

    fn as_point(&self) -> Option<&Point> {
        Some(self)
    }

    fn map_point_mut(&mut self, mut map: Box<dyn FnMut(&mut Point)>) {
        map(self)
    }
}

impl<'gl> NamedEntity for Point<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn set_similar_name(&mut self, name: &str) {
        self.name.set_similar_name(name)
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "position": {
                "x": self.position.translation.x,
                "y": self.position.translation.y,
                "z": self.position.translation.z
            },
            "name": self.name(),
        })
    }
}
