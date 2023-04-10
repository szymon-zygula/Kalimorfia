use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{
        ControlResult, DrawType, Drawable, Entity, NamedEntity, ReferentialDrawable,
        ReferentialEntity, ReferentialSceneEntity, ReferentialSceneObject, SceneObject,
    },
    point::Point,
    utils,
};
use crate::{
    camera::Camera,
    math::geometry::{bezier::BezierBSpline, curvable::Curvable},
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, mesh::LinesMesh, shader_manager::ShaderManager},
    repositories::{NameRepository, UniqueNameRepository},
    ui::{ordered_selector, single_selector},
};
use nalgebra::{Matrix4, Point2, Point3, Vector3};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

pub struct CubicSplineC2<'gl> {
    gl: &'gl glow::Context,
    mesh: RefCell<Option<LinesMesh<'gl>>>,
    deboor_polygon_mesh: RefCell<Option<LinesMesh<'gl>>>,
    bernstein_polygon_mesh: RefCell<Option<LinesMesh<'gl>>>,
    draw_deboor_polygon: bool,
    draw_bernstein_polygon: bool,
    show_bernstein_basis: bool,
    selected_bernstein_point: Option<usize>,
    points: Vec<usize>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
    last_camera: RefCell<Option<Camera>>,
    bernstein_points: Option<Vec<Point<'gl>>>,
    bspline: Option<BezierBSpline>,
}

impl<'gl> CubicSplineC2<'gl> {
    pub fn through_points(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        point_ids: Vec<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> Self {
        let mut created = Self {
            gl,
            points: point_ids,
            mesh: RefCell::new(None),
            deboor_polygon_mesh: RefCell::new(None),
            bernstein_polygon_mesh: RefCell::new(None),
            draw_deboor_polygon: false,
            draw_bernstein_polygon: false,
            show_bernstein_basis: false,
            selected_bernstein_point: None,
            name: ChangeableName::new("Cubic Spline C2", name_repo),
            last_camera: RefCell::new(None),
            bernstein_points: None,
            shader_manager,
            bspline: None,
        };

        created.recalculate_bspline(entities);
        created
    }

    fn bspline(
        point_ids: &[usize],
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> BezierBSpline {
        BezierBSpline::through_points(
            point_ids
                .iter()
                .map(|id| {
                    let p = entities[id].borrow().location().unwrap();
                    Point3::new(p.x as f64, p.y as f64, p.z as f64)
                })
                .collect(),
        )
    }

    fn generate_bernstein(
        gl: &'gl glow::Context,
        bspline: &BezierBSpline,
        shader_manager: &Rc<ShaderManager<'gl>>,
    ) -> Vec<Point<'gl>> {
        let name_repo: Rc<RefCell<dyn NameRepository>> =
            Rc::new(RefCell::new(UniqueNameRepository::new()));
        bspline
            .bernstein_points()
            .into_iter()
            .map(|p| {
                Point::with_position(
                    gl,
                    Point3::new(p.x as f32, p.y as f32, p.z as f32),
                    Rc::clone(&name_repo),
                    Rc::clone(shader_manager),
                )
            })
            .collect()
    }

    fn recalculate_bspline(
        &mut self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.selected_bernstein_point = None;
        if self.points.len() < 4 {
            self.bernstein_points = None;
            self.bspline = None;
            return;
        }

        let bspline = Self::bspline(&self.points, entities);
        self.bernstein_points = Some(Self::generate_bernstein(
            self.gl,
            &bspline,
            &self.shader_manager,
        ));
        self.bspline = Some(bspline);
    }

    fn set_new_bspline(
        &mut self,
        bspline: BezierBSpline,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        for (idx, deboor) in bspline.deboor_points().iter().enumerate() {
            let mut transform = LinearTransformEntity::new();
            transform.translation.translation =
                Vector3::new(deboor.x as f32, deboor.y as f32, deboor.z as f32);

            entities[&self.points[idx]]
                .borrow_mut()
                .set_model_transform(transform);
        }

        for (idx, bernstein) in bspline.bernstein_points().iter().enumerate() {
            let mut transform = LinearTransformEntity::new();
            transform.translation.translation =
                Vector3::new(bernstein.x as f32, bernstein.y as f32, bernstein.z as f32);

            SceneObject::set_model_transform(
                &mut self.bernstein_points.as_mut().unwrap()[idx],
                transform,
            );
        }

        self.bspline = Some(bspline);
        self.invalidate_mesh();
    }

    fn recalculate_mesh(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
    ) {
        if let Some(bspline) = &self.bspline {
            let samples = (utils::polygon_pixel_length(&self.points, entities, camera) * 0.5)
                .round() as usize;
            let (vertices, indices) = bspline.curve(samples);

            if vertices.is_empty() || indices.is_empty() {
                self.invalidate_mesh();
                return;
            }

            let mut mesh = LinesMesh::new(self.gl, vertices, indices);
            mesh.thickness(3.0);
            self.mesh.replace(Some(mesh));

            let mut bernstein_mesh = LinesMesh::strip(self.gl, bspline.bernstein_points_f32());
            bernstein_mesh.thickness(2.0);
            self.bernstein_polygon_mesh.replace(Some(bernstein_mesh));

            let mut deboor_mesh = LinesMesh::strip(self.gl, bspline.deboor_points_f32());
            deboor_mesh.thickness(1.0);
            self.deboor_polygon_mesh.replace(Some(deboor_mesh));
        } else {
            self.invalidate_mesh();
        }
    }

    fn invalidate_mesh(&self) {
        self.mesh.replace(None);
        self.deboor_polygon_mesh.replace(None);
        self.bernstein_polygon_mesh.replace(None);
    }

    fn is_mesh_valid(&self) -> bool {
        self.mesh.borrow().is_some()
            && self.deboor_polygon_mesh.borrow().is_some()
            && self.bernstein_polygon_mesh.borrow().is_some()
    }

    fn update_bernstein_from(
        &mut self,
        idx: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        let point_f64 =
            SceneObject::location(&self.bernstein_points.as_ref().unwrap()[idx]).unwrap();

        let new_bspline = self.bspline.as_ref().unwrap().modify_bernstein(
            idx,
            Point3::new(point_f64.x as f64, point_f64.y as f64, point_f64.z as f64),
        );

        self.set_new_bspline(new_bspline, entities);
    }
}

impl<'gl> ReferentialEntity<'gl> for CubicSplineC2<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("c2_spline");
        self.name_control_ui(ui);
        ui.checkbox("Draw de Boor polygon", &mut self.draw_deboor_polygon);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);
        ui.checkbox("Show Bernstein basis", &mut self.show_bernstein_basis);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);

        if self.show_bernstein_basis {
            let _token = ui.push_id("c2_bernstein_control");
            ui.separator();
            ui.text("Bernstein basis points");
            if let Some(bernstein_points) = &self.bernstein_points {
                let point_names: Vec<(usize, String)> = bernstein_points
                    .iter()
                    .enumerate()
                    .map(|(idx, point)| (idx, point.name()))
                    .collect();

                self.selected_bernstein_point = single_selector::single_selector(
                    ui,
                    &point_names,
                    self.selected_bernstein_point,
                );
            }

            let bernstein_changed = self
                .selected_bernstein_point
                .zip(self.bernstein_points.as_mut())
                .map_or(false, |(id, points)| points[id].control_ui(ui));

            if bernstein_changed {
                let idx = self.selected_bernstein_point.unwrap();
                self.update_bernstein_from(idx, entities);

                let mut modified: HashSet<usize> = self.points.iter().copied().collect();
                modified.insert(controller_id);
                return ControlResult {
                    modified,
                    ..Default::default()
                };
            }
        }

        if ordered_selector::changed(&self.points, &new_points) {
            utils::update_point_subs(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.recalculate_bspline(entities);
            self.invalidate_mesh();
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
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> bool {
        self.points.push(id);
        self.recalculate_bspline(entities);
        self.invalidate_mesh();
        true
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.recalculate_bspline(entities);
        self.invalidate_mesh();
    }

    fn notify_about_deletion(
        &mut self,
        deleted: &HashSet<usize>,
        remaining: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) {
        self.points.retain(|id| !deleted.contains(id));
        self.recalculate_bspline(remaining);
        self.invalidate_mesh();
    }
}

impl<'gl> ReferentialDrawable<'gl> for CubicSplineC2<'gl> {
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

        if let Some(((mesh, deboor_polygon_mesh), bernstein_polygon_mesh)) = self
            .mesh
            .borrow()
            .as_ref()
            .zip(self.deboor_polygon_mesh.borrow().as_ref())
            .zip(self.bernstein_polygon_mesh.borrow().as_ref())
        {
            mesh.draw();

            if self.draw_deboor_polygon {
                deboor_polygon_mesh.draw();
            }

            if self.draw_bernstein_polygon {
                bernstein_polygon_mesh.draw();
            }

            if self.show_bernstein_basis {
                if let Some(ref points) = self.bernstein_points {
                    for (idx, point) in points.iter().enumerate() {
                        let draw_type = if self.selected_bernstein_point.eq(&Some(idx))
                            && draw_type == DrawType::Selected
                        {
                            DrawType::SelectedVirtual
                        } else {
                            DrawType::Virtual
                        };

                        point.draw(camera, &Matrix4::identity(), draw_type);
                    }
                }
            }
        }
    }
}

impl<'gl> ReferentialSceneObject<'gl> for CubicSplineC2<'gl> {
    fn is_at_point(
        &mut self,
        point: Point2<f32>,
        projection_transform: &Matrix4<f32>,
        view_transform: &Matrix4<f32>,
        resolution: &glutin::dpi::PhysicalSize<u32>,
        _entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    ) -> (bool, f32) {
        if let Some(bernsteins) = &self.bernstein_points {
            for (idx, bernstein) in bernsteins.iter().enumerate() {
                let (is, val) = SceneObject::is_at_point(
                    bernstein,
                    point,
                    projection_transform,
                    view_transform,
                    resolution,
                );

                if is {
                    self.selected_bernstein_point = Some(idx);
                    return (is, val);
                }
            }
        }

        (false, 0.0)
    }

    fn set_ndc<'a>(
        &mut self,
        ndc: &Point2<f32>,
        camera: &Camera,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        controller_id: usize,
    ) -> ControlResult {
        if let Some(idx) = self.selected_bernstein_point {
            SceneObject::set_ndc(
                &mut self.bernstein_points.as_mut().unwrap()[idx],
                ndc,
                camera,
            );
            self.update_bernstein_from(idx, entities);
            let mut modified: HashSet<usize> = self.points.iter().copied().collect();
            modified.insert(controller_id);
            ControlResult {
                modified,
                ..Default::default()
            }
        } else {
            ControlResult::default()
        }
    }
}

impl<'gl> NamedEntity for CubicSplineC2<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }
}
