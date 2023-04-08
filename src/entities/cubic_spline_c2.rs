use super::{
    changeable_name::ChangeableName,
    entity::{
        DrawType, Drawable, NamedEntity, ReferentialDrawable, ReferentialEntity,
        ReferentialSceneEntity, SceneObject,
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
    ui::ordered_selector,
};
use nalgebra::{Matrix4, Point3};
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
        point_ids: &Vec<usize>,
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
        if self.points.len() < 4 {
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

    fn recalculate_mesh(
        &self,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        camera: &Camera,
    ) {
        if let Some(ref bspline) = self.bspline {
            let samples = (utils::polygon_pixel_length(&self.points, entities, camera) * 0.5)
                .round() as usize;
            let (vertices, indices) = bspline.curve(samples);

            if vertices.is_empty() || indices.is_empty() {
                self.invalidate_mesh();
                return;
            }

            self.mesh
                .replace(Some(LinesMesh::new(self.gl, vertices, indices)));

            self.bernstein_polygon_mesh.replace(Some(LinesMesh::strip(
                self.gl,
                bspline.bernstein_points_f32(),
            )));

            self.deboor_polygon_mesh
                .replace(Some(LinesMesh::strip(self.gl, bspline.deboor_points_f32())));
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
}

impl<'gl> ReferentialEntity<'gl> for CubicSplineC2<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        controller_id: usize,
        entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
        subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> HashSet<usize> {
        self.name_control_ui(ui);
        ui.checkbox("Draw de Boor polygon", &mut self.draw_deboor_polygon);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);
        ui.checkbox("Show Bernstein basis", &mut self.show_bernstein_basis);

        let points_names_selections = utils::segregate_points(entities, &self.points);

        let new_selection = ordered_selector::ordered_selector(ui, points_names_selections);
        let new_points = ordered_selector::selected_only(&new_selection);
        let changed = ordered_selector::changed(&self.points, &new_points);

        if changed {
            utils::update_point_subs(new_selection, controller_id, subscriptions);
            self.points = new_points;
            self.recalculate_bspline(entities);
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
                    for point in points {
                        point.draw(camera, &Matrix4::identity(), DrawType::Virtual);
                    }
                }
            }
        }
    }
}

impl<'gl> SceneObject for CubicSplineC2<'gl> {}

impl<'gl> NamedEntity for CubicSplineC2<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }
}
