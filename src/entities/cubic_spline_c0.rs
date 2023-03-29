use super::{
    changeable_name::ChangeableName,
    entity::{
        DrawType, NamedEntity, ReferentialDrawable, ReferentialEntity, ReferentialSceneEntity,
        SceneObject,
    },
    manager::EntityManager,
};
use crate::{
    camera::Camera,
    math::geometry::{self, curvable::Curvable},
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, mesh::LinesMesh},
    repositories::NameRepository,
    ui::ordered_selector::ordered_selelector,
};
use nalgebra::{Matrix4, Point3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
    rc::Rc,
};

pub struct CubicSplineC0<'gl> {
    gl: &'gl glow::Context,
    mesh: RefCell<Option<LinesMesh<'gl>>>,
    polygon_mesh: RefCell<Option<LinesMesh<'gl>>>,
    draw_polygon: bool,
    points: Vec<usize>,
    gl_program: GlProgram<'gl>,
    name: ChangeableName,
    last_camera: RefCell<Option<Camera>>,
}

impl<'gl> CubicSplineC0<'gl> {
    pub fn through_points(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        point_ids: Vec<usize>,
        entity_manager: &RefCell<EntityManager<'gl>>,
    ) -> CubicSplineC0<'gl> {
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

        Self {
            gl,
            points: point_ids,
            mesh: RefCell::new(None),
            polygon_mesh: RefCell::new(None),
            draw_polygon: false,
            gl_program,
            name: ChangeableName::new("Cubic Spline C0", name_repo),
            last_camera: RefCell::new(None),
        }
    }

    fn spline_mesh(
        point_ids: &Vec<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> (Vec<Point3<f32>>, Vec<u32>) {
        let mut points = Vec::with_capacity(point_ids.len());

        for &id in point_ids {
            let p = entities[&id].borrow().location().unwrap();
            points.push(Point3::new(p.x as f64, p.y as f64, p.z as f64));
        }

        let spline = geometry::bezier::CubicSplineC0::through_points(points);
        let vertices = spline.curve(100); // TODO: make adaptative

        let mut indices = Vec::with_capacity(200);
        for i in 0..99 {
            indices.push(i);
            indices.push(i + 1);
        }

        (vertices, indices)
    }

    fn polygon_mesh(
        point_ids: &Vec<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> (Vec<Point3<f32>>, Vec<u32>) {
        let mut points = Vec::with_capacity(point_ids.len());

        for &id in point_ids {
            points.push(entities[&id].borrow().location().unwrap());
        }

        let mut indices = Vec::with_capacity(points.len() * 2);
        for i in 0..(points.len() as u32 - 1) {
            indices.push(i);
            indices.push(i + 1);
        }

        (points, indices)
    }

    fn recalculate_mesh(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
    ) {
        if self.points.is_empty() {
            self.invalidate_mesh();
        } else {
            let (vertices, indices) = Self::spline_mesh(&self.points, entities);
            self.mesh
                .replace(Some(LinesMesh::new(self.gl, vertices, indices)));
            let (vertices, indices) = Self::polygon_mesh(&self.points, entities);
            self.polygon_mesh
                .replace(Some(LinesMesh::new(self.gl, vertices, indices)));
        }
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
    ) -> HashSet<usize> {
        self.name_control_ui(ui);
        ui.checkbox("Draw polygon", &mut self.draw_polygon);

        let selected = self
            .points
            .iter()
            .map(|id| (*id, entities[id].borrow().name(), true));

        let not_selected = entities
            .iter()
            .filter(|(id, entity)| {
                !self.points.contains(id)
                    && entity
                        .try_borrow()
                        .map_or(false, |entity| entity.is_single_point())
            })
            .map(|(id, entity)| (*id, entity.borrow().name(), false));

        let points_names_selections = selected.chain(not_selected).collect();

        let new_selection = ordered_selelector(ui, points_names_selections);
        let new_points: Vec<usize> = new_selection
            .iter()
            .filter(|(_, selected)| *selected)
            .map(|(id, _)| *id)
            .collect();

        let changed = self.points.iter().ne(new_points.iter());

        if changed {
            for (id, selected) in new_selection {
                if selected {
                    subscriptions.get_mut(&controller_id).unwrap().insert(id);
                } else {
                    subscriptions.get_mut(&controller_id).unwrap().remove(&id);
                }
            }

            self.points = new_points;
            self.invalidate_mesh();
        }

        if changed {
            HashSet::from([controller_id])
        } else {
            HashSet::new()
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
        if self.last_camera.borrow().as_ref().eq(&Some(camera)) {
            self.recalculate_mesh(entities, camera);
        } else {
            self.last_camera.replace(Some(camera.clone()));
        }

        if !self.is_mesh_valid() {
            self.recalculate_mesh(entities, camera);
        }

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        self.gl_program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

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
