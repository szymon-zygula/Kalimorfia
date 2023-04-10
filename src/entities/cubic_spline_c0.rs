use super::{
    changeable_name::ChangeableName,
    entity::{
        ControlResult, DrawType, NamedEntity, ReferentialDrawable, ReferentialEntity,
        ReferentialSceneEntity, SceneObject,
    },
    utils,
};
use crate::{
    camera::Camera,
    math::geometry::{self, curvable::Curvable},
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, mesh::LinesMesh, shader_manager::ShaderManager},
    repositories::NameRepository,
    ui::ordered_selector,
};
use nalgebra::{Matrix4, Point3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

pub struct CubicSplineC0<'gl> {
    gl: &'gl glow::Context,
    mesh: RefCell<Option<LinesMesh<'gl>>>,
    polygon_mesh: RefCell<Option<LinesMesh<'gl>>>,
    draw_polygon: bool,
    points: Vec<usize>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
    last_camera: RefCell<Option<Camera>>,
}

impl<'gl> CubicSplineC0<'gl> {
    pub fn through_points(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        point_ids: Vec<usize>,
    ) -> Self {
        Self {
            gl,
            points: point_ids,
            mesh: RefCell::new(None),
            polygon_mesh: RefCell::new(None),
            draw_polygon: false,
            shader_manager,
            name: ChangeableName::new("Cubic Spline C0", name_repo),
            last_camera: RefCell::new(None),
        }
    }

    fn spline_mesh(
        point_ids: &Vec<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        samples: u32,
    ) -> (Vec<Point3<f32>>, Vec<u32>) {
        let mut points = Vec::with_capacity(point_ids.len());

        for &id in point_ids {
            let p = entities[&id].borrow().location().unwrap();
            points.push(Point3::new(p.x as f64, p.y as f64, p.z as f64));
        }

        let spline = geometry::bezier::BezierCubicSplineC0::through_points(points);
        spline.curve(samples as usize)
    }

    fn polygon_mesh(
        &self,
        point_ids: &Vec<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> LinesMesh<'gl> {
        let mut points = Vec::with_capacity(point_ids.len());

        for &id in point_ids {
            points.push(entities[&id].borrow().location().unwrap());
        }

        let mut mesh = LinesMesh::strip(self.gl, points);
        mesh.thickness(2.0);
        mesh
    }

    fn recalculate_mesh(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
    ) {
        if self.points.is_empty() {
            self.invalidate_mesh();
            return;
        }

        let (vertices, indices) = Self::spline_mesh(
            &self.points,
            entities,
            (utils::polygon_pixel_length(&self.points, entities, camera) * 0.5).round() as u32,
        );

        if vertices.is_empty() || indices.is_empty() {
            self.invalidate_mesh();
            return;
        }

        let mut mesh = LinesMesh::new(self.gl, vertices, indices);
        mesh.thickness(3.0);
        self.mesh.replace(Some(mesh));
        let polygon_mesh = self.polygon_mesh(&self.points, entities);
        self.polygon_mesh.replace(Some(polygon_mesh));
    }

    fn invalidate_mesh(&self) {
        self.mesh.replace(None);
        self.polygon_mesh.replace(None);
    }

    fn is_mesh_valid(&self) -> bool {
        self.mesh.borrow().is_some() && self.polygon_mesh.borrow().is_some()
    }
}

impl<'gl> ReferentialEntity<'gl> for CubicSplineC0<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        self.name_control_ui(ui);
        ui.checkbox("Draw polygon", &mut self.draw_polygon);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);
        let changed = ordered_selector::changed(&self.points, &new_points);

        if changed {
            utils::update_point_subs(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.invalidate_mesh();
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

    fn add_point(
        &mut self,
        id: usize,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> bool {
        self.points.push(id);
        self.invalidate_mesh();
        true
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.invalidate_mesh();
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        _remaining: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.points.retain(|id| !deleted.contains(id));
        self.invalidate_mesh();
    }
}

impl<'gl> ReferentialDrawable<'gl> for CubicSplineC0<'gl> {
    fn draw_referential(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
        premul: &Matrix4<f32>,
        draw_type: DrawType,
    ) {
        if !self.last_camera.borrow().as_ref().eq(&Some(camera)) {
            self.invalidate_mesh();
            self.last_camera.replace(Some(camera.clone()));
        }

        if !self.is_mesh_valid() {
            self.recalculate_mesh(entities, camera);
        }

        let program = self.shader_manager.program("spline");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));

        if let Some((mesh, polygon_mesh)) = self
            .mesh
            .borrow()
            .as_ref()
            .zip(self.polygon_mesh.borrow().as_ref())
        {
            mesh.draw();

            if self.draw_polygon {
                polygon_mesh.draw();
            }
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
}
