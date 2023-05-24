use super::{
    changeable_name::ChangeableName,
    entity::{
        ControlResult, DrawType, EntityCollection, NamedEntity, ReferentialDrawable,
        ReferentialEntity, SceneObject,
    },
    utils,
};
use crate::{
    camera::Camera,
    math::geometry,
    primitives::color::Color,
    render::{
        bezier_mesh::BezierMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
    ui::ordered_selector,
};
use nalgebra::{Matrix4, Point3};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct CubicSplineC0<'gl> {
    gl: &'gl glow::Context,
    mesh: RefCell<BezierMesh<'gl>>,
    polygon_mesh: RefCell<LinesMesh<'gl>>,
    draw_polygon: bool,
    points: Vec<usize>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
}

impl<'gl> CubicSplineC0<'gl> {
    pub fn through_points(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        point_ids: Vec<usize>,
        entities: &EntityCollection<'gl>,
    ) -> Self {
        Self {
            gl,
            mesh: RefCell::new(Self::curve_mesh(gl, &point_ids, entities)),
            polygon_mesh: RefCell::new(Self::polygon_mesh(gl, &point_ids, entities)),
            points: point_ids,
            draw_polygon: false,
            shader_manager,
            name: ChangeableName::new("Cubic spline C0", name_repo),
        }
    }

    fn polygon_mesh(
        gl: &'gl glow::Context,
        point_ids: &[usize],
        entities: &EntityCollection<'gl>,
    ) -> LinesMesh<'gl> {
        if point_ids.is_empty() {
            return LinesMesh::empty(gl);
        }

        let mut points = Vec::with_capacity(point_ids.len());

        for &id in point_ids {
            points.push(entities[&id].borrow().location().unwrap());
        }

        let mut mesh = LinesMesh::strip(gl, points);
        mesh.thickness(2.0);
        mesh
    }

    fn curve_mesh(
        gl: &'gl glow::Context,
        point_ids: &[usize],
        entities: &EntityCollection<'gl>,
    ) -> BezierMesh<'gl> {
        if point_ids.is_empty() {
            return BezierMesh::empty(gl);
        }

        let points = point_ids
            .iter()
            .map(|id| {
                let p = entities[id].borrow().location().unwrap();
                Point3::new(p.x as f64, p.y as f64, p.z as f64)
            })
            .collect();

        let spline = geometry::bezier::BezierCubicSplineC0::through_points(points);
        let mut mesh = BezierMesh::new(gl, spline);
        mesh.thickness(3.0);
        mesh
    }

    fn recalculate_mesh(&self, entities: &EntityCollection<'gl>) {
        if self.points.is_empty() {
            self.mesh.replace(BezierMesh::empty(self.gl));
            self.polygon_mesh.replace(LinesMesh::empty(self.gl));
            return;
        }

        let mesh = Self::curve_mesh(self.gl, &self.points, entities);
        self.mesh.replace(mesh);

        let polygon_mesh = Self::polygon_mesh(self.gl, &self.points, entities);
        self.polygon_mesh.replace(polygon_mesh);
    }

    fn draw_polygon(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("spline");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));

        self.polygon_mesh.borrow().draw();
    }

    fn draw_curve(
        &self,
        entities: &EntityCollection<'gl>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        let program = self.shader_manager.program("bezier");
        let polygon_pixel_length = utils::polygon_pixel_length(&self.points, entities, camera);
        // This is not quite right when one of the segments is just a single point, but it's good
        // enough
        let segment_pixel_count = polygon_pixel_length / (self.points.len() / 3 + 1) as f32;

        self.mesh.borrow().draw_with_program(
            program,
            camera,
            segment_pixel_count,
            premul,
            &Color::for_draw_type(&draw_type),
        )
    }
}

impl<'gl> ReferentialEntity<'gl> for CubicSplineC0<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &EntityCollection<'gl>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        self.name_control_ui(ui);
        ui.checkbox("Draw polygon", &mut self.draw_polygon);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);
        let changed = ordered_selector::changed(&self.points, &new_points);

        if changed {
            utils::update_point_subscriptions(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.recalculate_mesh(entities);
        }

        if changed {
            ControlResult {
                modified: HashSet::from([controller_id]),
                ..Default::default()
            }
        } else {
            ControlResult::default()
        }
    }

    fn add_point(&mut self, id: usize, entities: &EntityCollection<'gl>) -> bool {
        self.points.push(id);
        self.recalculate_mesh(entities);
        true
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &EntityCollection<'gl>,
    ) {
        self.recalculate_mesh(entities);
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        remaining: &EntityCollection<'gl>,
    ) {
        self.points.retain(|id| !deleted.contains(id));
        self.recalculate_mesh(remaining);
    }
}

impl<'gl> ReferentialDrawable<'gl> for CubicSplineC0<'gl> {
    fn draw_referential(
        &self,
        entities: &EntityCollection<'gl>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        self.draw_curve(entities, camera, premul, draw_type);

        if self.draw_polygon {
            self.draw_polygon(camera, premul, draw_type);
        }
    }
}

impl<'gl> SceneObject for CubicSplineC0<'gl> {}

impl<'gl> NamedEntity for CubicSplineC0<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "objectType": "bezierC0",
            "controlPoints": utils::control_points_json(&self.points),
            "name": self.name()
        })
    }
}
