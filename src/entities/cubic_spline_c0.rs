use super::{
    changeable_name::ChangeableName,
    entity::{Drawable, NamedEntity, ReferentialEntity, ReferentialSceneEntity, SceneObject},
    manager::EntityManager,
};
use crate::{
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
    mesh: LinesMesh<'gl>,
    polygon_mesh: LinesMesh<'gl>,
    draw_polygon: bool,
    points: Vec<usize>,
    gl_program: GlProgram<'gl>,
    name: ChangeableName,
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

        let mut spline = Self {
            gl,
            points: point_ids,
            mesh: LinesMesh::empty(gl),
            polygon_mesh: LinesMesh::empty(gl),
            draw_polygon: false,
            gl_program,
            name: ChangeableName::new("Cubic Spline C0", name_repo),
        };

        spline.recalculate_mesh(entity_manager.borrow().entities());

        spline
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
        &mut self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        if self.points.is_empty() {
            self.mesh = LinesMesh::empty(self.gl);
            self.polygon_mesh = LinesMesh::empty(self.gl);
        } else {
            let (vertices, indices) = Self::spline_mesh(&self.points, entities);
            self.mesh = LinesMesh::new(self.gl, vertices, indices);
            let (vertices, indices) = Self::polygon_mesh(&self.points, entities);
            self.polygon_mesh = LinesMesh::new(self.gl, vertices, indices);
        }
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
                // Subscriptions
            }

            self.points = new_points;
            self.recalculate_mesh(entities);
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
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> bool {
        self.points.push(id);
        self.recalculate_mesh(entities);
        true
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.recalculate_mesh(entities);
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        remaining: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.points.retain(|id| !deleted.contains(id));
        self.recalculate_mesh(remaining);
    }
}

impl<'gl> Drawable for CubicSplineC0<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", Matrix4::identity().as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("projection_transform", projection_transform.as_slice());

        self.mesh.draw();

        if self.draw_polygon {
            self.polygon_mesh.draw();
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
