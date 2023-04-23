use crate::{
    camera::Camera,
    entities::{
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, EntityCollection, NamedEntity, ReferentialDrawable,
            ReferentialEntity, SceneObject,
        },
        point::Point,
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
    repositories::{NameRepository, UniqueNameRepository},
    ui::{ordered_selector, single_selector},
};
use nalgebra::{Matrix4, Point3};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct InterpolatingSpline<'gl> {
    gl: &'gl glow::Context,

    mesh: RefCell<BezierMesh<'gl>>,
    interpolating_polygon_mesh: RefCell<LinesMesh<'gl>>,
    deboor_polygon_mesh: RefCell<LinesMesh<'gl>>,
    bernstein_polygon_mesh: RefCell<LinesMesh<'gl>>,

    draw_interpolating_polygon: bool,
    draw_deboor_polygon: bool,
    draw_bernstein_polygon: bool,

    points: Vec<usize>,
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
        let spline = Self {
            gl,
            mesh: RefCell::new(BezierMesh::empty(gl)),

            interpolating_polygon_mesh: RefCell::new(LinesMesh::empty(gl)),
            deboor_polygon_mesh: RefCell::new(LinesMesh::empty(gl)),
            bernstein_polygon_mesh: RefCell::new(LinesMesh::empty(gl)),

            draw_interpolating_polygon: false,
            draw_deboor_polygon: false,
            draw_bernstein_polygon: false,

            points,
            shader_manager,
            name: ChangeableName::new("Interpolating Spline", name_repo),
        };

        spline.recalculate_mesh(entities);
        spline
    }

    fn recalculate_mesh(&self, entities: &EntityCollection<'gl>) {
        if self.points.len() <= 1 {
            self.mesh.replace(BezierMesh::empty(self.gl));
            self.interpolating_polygon_mesh
                .replace(LinesMesh::empty(self.gl));
            self.deboor_polygon_mesh.replace(LinesMesh::empty(self.gl));
            self.bernstein_polygon_mesh
                .replace(LinesMesh::empty(self.gl));
            return;
        }

        let points = &self
            .points
            .iter()
            .map(|id| entities[id].borrow().location().unwrap())
            .map(|p| Point3::new(p.x as f64, p.y as f64, p.z as f64))
            .collect::<Vec<_>>();

        if self.points.len() == 2 {
            let spline = BezierCubicSplineC0::through_points(points.clone());
            let mut mesh = BezierMesh::new(self.gl, spline);
            mesh.thickness(3.0);
            self.mesh.replace(mesh);

            let points32 = points
                .iter()
                .copied()
                .map(math::utils::point_64_to_32)
                .collect();

            self.bernstein_polygon_mesh
                .replace(LinesMesh::strip(self.gl, points32));
            return;
        }

        let bernstein_tuples = interpolating_spline_c2(points);

        let mut bernstein_points: Vec<_> = bernstein_tuples
            .iter()
            .copied()
            .flat_map(|(b0, b1, b2, _)| [b0, b1, b2])
            .collect();
        bernstein_points.push(bernstein_tuples.last().unwrap().3);

        let spline = BezierCubicSplineC0::through_points(bernstein_points.clone());

        let bernstein_points = bernstein_points
            .iter()
            .copied()
            .map(math::utils::point_64_to_32)
            .collect();

        let mut mesh = BezierMesh::new(self.gl, spline);
        mesh.thickness(3.0);
        self.mesh.replace(mesh);

        self.bernstein_polygon_mesh
            .replace(LinesMesh::strip(self.gl, bernstein_points));
    }

    fn draw_curve(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("bezier");
        let polygon_pixel_length =
            utils::polygon_pixel_length_direct(&Vec::new() /* TODO */, camera);

        let segment_pixel_count = polygon_pixel_length / (self.points.len() / 3 + 1) as f32;
        self.mesh.borrow().draw_with_program(
            program,
            camera,
            segment_pixel_count,
            premul,
            &Color::for_draw_type(&draw_type),
        );

        self.mesh.borrow().draw();
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
        ui.checkbox("Draw interpolating", &mut self.draw_interpolating_polygon);
        ui.checkbox("Draw de Boor polygon", &mut self.draw_deboor_polygon);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);

        if ordered_selector::changed(&self.points, &new_points) {
            utils::update_point_subscriptions(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.recalculate_mesh(entities);

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
            self.interpolating_polygon_mesh.borrow().draw();
        }

        if self.draw_deboor_polygon {
            self.deboor_polygon_mesh.borrow().draw();
        }

        if self.draw_bernstein_polygon {
            self.bernstein_polygon_mesh.borrow().draw();
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
}
