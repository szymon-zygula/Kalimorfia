use super::{
    changeable_name::ChangeableName,
    entity::{
        ControlResult, DrawType, Drawable, EntityCollection, NamedEntity, ReferentialEntity,
        SceneObject,
    },
};
use crate::{
    camera::Camera,
    math::{geometry::intersection::Intersection, utils::point_64_to_32},
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, mesh::LinesMesh, shader_manager::ShaderManager},
    repositories::NameRepository,
};
use nalgebra::Matrix4;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct IntersectionCurve<'gl> {
    gl: &'gl glow::Context,
    mesh: LinesMesh<'gl>,
    intersection: Intersection,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
}

impl<'gl> IntersectionCurve<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        entities: &EntityCollection<'gl>,
        intersection: Intersection,
    ) -> Self {
        let mut points: Vec<_> = intersection
            .points
            .iter()
            .map(|point| point_64_to_32(point.point))
            .collect();

        if intersection.looped {
            points.push(points[0]);
        }

        let mut mesh = LinesMesh::strip(gl, points);
        mesh.thickness(3.0);

        Self {
            gl,
            mesh,
            intersection,
            shader_manager,
            name: ChangeableName::new("Intersection Curve", name_repo),
        }
    }
}

impl<'gl> ReferentialEntity<'gl> for IntersectionCurve<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        self.name_control_ui(ui);
        // TODO: show textures, enable trimming
        ControlResult::default()
    }

    fn notify_about_reindexing(
        &mut self,
        _changes: &HashMap<usize, usize>,
        _entities: &EntityCollection<'gl>,
    ) {
        todo!("Reindex surfaces for intersection")
    }

    fn allow_deletion(&self, _deleted: &HashSet<usize>) -> bool {
        // Refuse deletion of intersected surfaces
        false
    }
}

impl<'gl> Drawable for IntersectionCurve<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("line_mesh");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        let color = match draw_type {
            DrawType::Regular => Color::lblue(),
            _ => Color::for_draw_type(&draw_type),
        };

        program.uniform_color("color", &color);

        self.mesh.draw();
    }
}

impl<'gl> SceneObject for IntersectionCurve<'gl> {}

impl<'gl> NamedEntity for IntersectionCurve<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }

    fn set_similar_name(&mut self, name: &str) {
        self.name.set_similar_name(name)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "objectType": "intersectionCurve",
            "name": self.name()
        })
    }
}
