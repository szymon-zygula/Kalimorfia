use crate::{
    camera::Camera,
    entities::{
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, EntityCollection, NamedEntity, ReferentialDrawable,
            ReferentialEntity, SceneObject,
        },
        utils,
    },
    math::{
        self,
        geometry::{bezier::BezierCubicSplineC0, interpolating_spline::interpolating_spline_c2},
    },
    primitives::color::Color,
    render::{
        bezier_mesh::BezierMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
    ui::ordered_selector,
};
use itertools::Itertools;
use nalgebra::{Matrix4, Point3};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct InterpolatingSpline<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierMesh<'gl>,
    interpolating_polygon_mesh: LinesMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_interpolating_polygon: bool,
    draw_bernstein_polygon: bool,

    points: Vec<usize>,
    bernstein_points: Vec<Point3<f32>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
}

impl<'gl> InterpolatingSpline<'gl> {
    pub fn through_points(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        points: Vec<usize>,
        entities: &EntityCollection<'gl>,
    ) -> Self {
        let mut spline = Self {
            gl,
            mesh: BezierMesh::empty(gl),

            interpolating_polygon_mesh: LinesMesh::empty(gl),
            bernstein_polygon_mesh: LinesMesh::empty(gl),

            draw_interpolating_polygon: false,
            draw_bernstein_polygon: false,

            points,
            bernstein_points: Vec::new(),
            shader_manager,
            name: ChangeableName::new("Interpolating Spline", name_repo),
        };

        spline.recalculate_bernstein(entities);
        spline
    }

    fn unique_point_sequence(&self, entities: &EntityCollection<'gl>) -> Vec<Point3<f64>> {
        self.points
            .iter()
            .map(|id| entities[id].borrow().location().unwrap())
            .map(|p| Point3::new(p.x as f64, p.y as f64, p.z as f64))
            .dedup()
            .collect()
    }

    fn set_interpolating_polygon_mesh(&mut self, points: Vec<Point3<f32>>) {
        let mut interpolating_mesh = LinesMesh::strip(self.gl, points);
        interpolating_mesh.thickness(2.0);
        self.interpolating_polygon_mesh = interpolating_mesh;
    }

    fn set_bernstein_polygon_mesh(&mut self, points: Vec<Point3<f32>>) {
        self.bernstein_polygon_mesh = LinesMesh::strip(self.gl, points);
    }

    fn recalculate_bernstein(&mut self, entities: &EntityCollection<'gl>) {
        let points = self.unique_point_sequence(entities);

        self.bernstein_points = match &points[..] {
            &[] | &[_] => Vec::new(),
            &[p0, p1] => [p0, p0, p1, p1]
                .into_iter()
                .map(math::utils::point_64_to_32)
                .collect(),
            points => {
                let bernstein_tuples = interpolating_spline_c2(points);

                let mut bernstein_points: Vec<_> = bernstein_tuples
                    .iter()
                    .copied()
                    .flat_map(|(b0, b1, b2, _)| [b0, b1, b2])
                    .collect();
                bernstein_points.push(bernstein_tuples.last().unwrap().3);

                bernstein_points
                    .iter()
                    .copied()
                    .map(math::utils::point_64_to_32)
                    .collect()
            }
        };

        self.recalculate_mesh(entities);
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let points = self.unique_point_sequence(entities);

        let points32: Vec<_> = points
            .iter()
            .copied()
            .map(math::utils::point_64_to_32)
            .collect();

        if points.len() <= 1 {
            self.mesh = BezierMesh::empty(self.gl);
            self.interpolating_polygon_mesh = LinesMesh::empty(self.gl);
            self.bernstein_polygon_mesh = LinesMesh::empty(self.gl);
            return;
        }

        if points.len() == 2 {
            let spline = BezierCubicSplineC0::through_points(points);
            let mut mesh = BezierMesh::new(self.gl, spline);
            mesh.thickness(3.0);
            self.mesh = mesh;

            self.set_interpolating_polygon_mesh(points32.clone());
            self.set_bernstein_polygon_mesh(points32);

            return;
        }

        let spline = BezierCubicSplineC0::through_points(
            self.bernstein_points
                .iter()
                .copied()
                .map(math::utils::point_32_to_64)
                .collect(),
        );

        let mut mesh = BezierMesh::new(self.gl, spline);
        mesh.thickness(3.0);
        self.mesh = mesh;

        self.set_interpolating_polygon_mesh(points32);
        self.set_bernstein_polygon_mesh(self.bernstein_points.clone());
    }

    fn draw_curve(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("bezier");
        let polygon_pixel_length =
            utils::polygon_pixel_length_direct(&self.bernstein_points, camera);

        let segment_pixel_count = polygon_pixel_length / (self.points.len() / 3 + 1) as f32;
        self.mesh.draw_with_program(
            program,
            camera,
            segment_pixel_count,
            premul,
            &Color::for_draw_type(&draw_type),
        );

        self.mesh.draw();
    }
}

impl<'gl> ReferentialEntity<'gl> for InterpolatingSpline<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &EntityCollection<'gl>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("interpolating_spline");
        self.name_control_ui(ui);
        ui.checkbox(
            "Draw interpolating polygon",
            &mut self.draw_interpolating_polygon,
        );
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);

        if ordered_selector::changed(&self.points, &new_points) {
            utils::update_point_subscriptions(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.recalculate_bernstein(entities);

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
        self.recalculate_bernstein(entities);
        true
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &EntityCollection<'gl>,
    ) {
        self.recalculate_bernstein(entities);
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        remaining: &EntityCollection<'gl>,
    ) {
        self.points.retain(|id| !deleted.contains(id));
        self.recalculate_bernstein(remaining);
    }

    fn notify_about_reindexing(
        &mut self,
        changes: &HashMap<usize, usize>,
        entities: &EntityCollection<'gl>,
    ) {
        for old_id in &mut self.points {
            if let Some(&new_id) = changes.get(old_id) {
                *old_id = new_id;
            }
        }

        self.recalculate_bernstein(entities);
    }
}

impl<'gl> ReferentialDrawable<'gl> for InterpolatingSpline<'gl> {
    fn draw_referential(
        &self,
        _entities: &EntityCollection<'gl>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        self.draw_curve(camera, premul, draw_type);

        let program = self.shader_manager.program("spline");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));

        if self.draw_interpolating_polygon {
            self.interpolating_polygon_mesh.draw();
        }

        if self.draw_bernstein_polygon {
            self.bernstein_polygon_mesh.draw();
        }
    }
}

impl<'gl> SceneObject for InterpolatingSpline<'gl> {}

impl<'gl> NamedEntity for InterpolatingSpline<'gl> {
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
            "objectType": "interpolatedC2",
            "name": self.name(),
            "controlPoints": utils::control_points_json(&self.points)
        })
    }
}
