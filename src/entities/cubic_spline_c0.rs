use super::{
    changeable_name::ChangeableName,
    entity::{
        DrawType, NamedEntity, ReferentialDrawable, ReferentialEntity, ReferentialSceneEntity,
        SceneObject,
    },
};
use crate::{
    camera::Camera,
    math::geometry::{self, curvable::Curvable},
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, mesh::LinesMesh, shader_manager::ShaderManager},
    repositories::NameRepository,
    ui::ordered_selector::ordered_selelector,
};
use nalgebra::{Matrix4, Point3, Vector2};
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
    ) -> CubicSplineC0<'gl> {
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

        let spline = geometry::bezier::CubicSplineC0::through_points(points);
        let (vertices, indices) = spline.curve(samples as usize);

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
            let (vertices, indices) = Self::spline_mesh(
                &self.points,
                entities,
                (self.polygon_pixel_length(entities, camera) * 0.5).round() as u32,
            );

            if vertices.is_empty() || indices.is_empty() {
                self.invalidate_mesh();
                return;
            }

            self.mesh
                .replace(Some(LinesMesh::new(self.gl, vertices, indices)));
            let (vertices, indices) = Self::polygon_mesh(&self.points, entities);
            self.polygon_mesh
                .replace(Some(LinesMesh::new(self.gl, vertices, indices)));
        }
    }

    fn polygon_pixel_length(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
    ) -> f32 {
        let mut sum = 0.0;
        for i in 1..self.points.len() {
            let point1 = entities[&self.points[i - 1]].borrow().location().unwrap();
            let point2 = entities[&self.points[i]].borrow().location().unwrap();

            let point1 =
                camera.projection_transform() * camera.view_transform() * point1.to_homogeneous();
            let point2 =
                camera.projection_transform() * camera.view_transform() * point2.to_homogeneous();

            let diff = Point3::from_homogeneous(point1)
                .unwrap_or(Point3::origin())
                .xy()
                - Point3::from_homogeneous(point2)
                    .unwrap_or(Point3::origin())
                    .xy();

            let clamped_point = Vector2::new(diff.x.clamp(-1.0, 1.0), diff.y.clamp(-1.0, 1.0));
            sum += clamped_point
                .component_mul(&Vector2::new(
                    0.5 * camera.window_size.width as f32,
                    0.5 * camera.window_size.height as f32,
                ))
                .norm();
        }

        sum
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
