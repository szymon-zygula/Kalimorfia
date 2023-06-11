use crate::{
    camera::Camera,
    entities::{
        bezier_surface_args::*,
        bezier_utils::*,
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, Drawable, EntityCollection, NamedEntity, ReferentialEntity,
            SceneObject,
        },
        utils,
    },
    graph::C0Edge,
    math::geometry::{parametric_form::DifferentialParametricForm, surfaces::SurfaceC0},
    render::{
        bezier_surface_mesh::BezierSurfaceMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use nalgebra::Matrix4;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct BezierSurfaceC0<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierSurfaceMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_bernstein_polygon: bool,

    points: Vec<Vec<usize>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    pub u_patch_divisions: u32,
    pub v_patch_divisions: u32,

    surface: SurfaceC0,

    is_cylinder: bool,

    gk_mode: bool,
    wireframe: bool,
}

impl<'gl> BezierSurfaceC0<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        points: Vec<Vec<usize>>,
        entities: &EntityCollection<'gl>,
        args: BezierSurfaceArgs,
    ) -> Self {
        let is_cylinder = matches!(args, BezierSurfaceArgs::Cylinder(..));
        let bezier_surface = create_bezier_surface(&points, entities, is_cylinder);

        let mut surface = Self {
            gl,
            mesh: BezierSurfaceMesh::empty(gl),
            points,
            bernstein_polygon_mesh: grid_mesh(gl, bezier_surface.grid()),
            draw_bernstein_polygon: false,
            name: ChangeableName::new("Bezier Surface C0", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
            surface: SurfaceC0::null(),
            is_cylinder,
            gk_mode: false,
            wireframe: true,
        };

        surface.recalculate_mesh(entities);
        surface
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let bezier_surface = create_bezier_surface(&self.points, entities, self.is_cylinder);
        self.surface =
            SurfaceC0::from_bezier_surface(bezier_surface.clone(), self.is_cylinder, false);
        self.mesh = BezierSurfaceMesh::new(self.gl, bezier_surface.clone());
        self.bernstein_polygon_mesh = grid_mesh(self.gl, bezier_surface.grid());
    }

    fn u_patches(&self) -> usize {
        if self.is_cylinder {
            self.points.len() / 3
        } else {
            (self.points.len() - 1) / 3
        }
    }

    fn v_patches(&self) -> usize {
        self.points.first().map_or(0, |first| (first.len() - 1) / 3)
    }

    fn patch_control_points(&self, patch_u: usize, patch_v: usize) -> Vec<usize> {
        let mut points = Vec::new();

        for v in 0..4 {
            for u in 0..4 {
                points.push(self.points[(patch_u * 3 + u) % self.points.len()][patch_v * 3 + v]);
            }
        }

        points
    }

    fn json_patches(&self) -> Vec<serde_json::Value> {
        let mut patches = Vec::new();

        let u_patches = self.u_patches();
        let v_patches = self.v_patches();

        for patch_v in 0..v_patches {
            for patch_u in 0..u_patches {
                patches.push(serde_json::json!({
                    "objectType": "bezierPatchC0",
                    "name": "patch",
                    "controlPoints": utils::control_points_json(
                        &self.patch_control_points(patch_u, patch_v)
                    ),
                    "samples": {
                        "x": self.u_patch_divisions,
                        "y": self.v_patch_divisions
                    }
                }));
            }
        }

        patches
    }

    ///
    /// *+++     &***
    /// *  #  => &  +
    /// *  #  => &  +
    /// &&&#     ###+
    ///
    fn rotate_patch(patch: &[[usize; 4]; 4]) -> [[usize; 4]; 4] {
        [
            [patch[3][0], patch[2][0], patch[1][0], patch[0][0]],
            [patch[3][1], patch[2][1], patch[1][1], patch[0][1]],
            [patch[3][2], patch[2][2], patch[1][2], patch[0][2]],
            [patch[3][3], patch[2][3], patch[1][3], patch[0][3]],
        ]
    }

    fn patch(&self, patch_u: usize, patch_v: usize) -> [[usize; 4]; 4] {
        let u = patch_u * 3;
        let v = patch_v * 3;

        [
            [
                self.points[u][v],
                self.points[u][v + 1],
                self.points[u][v + 2],
                self.points[u][v + 3],
            ],
            [
                self.points[u + 1][v],
                self.points[u + 1][v + 1],
                self.points[u + 1][v + 2],
                self.points[u + 1][v + 3],
            ],
            [
                self.points[u + 2][v],
                self.points[u + 2][v + 1],
                self.points[u + 2][v + 2],
                self.points[u + 2][v + 3],
            ],
            if self.is_cylinder && patch_u == self.u_patches() - 1 {
                [
                    self.points[0][v],
                    self.points[0][v + 1],
                    self.points[0][v + 2],
                    self.points[0][v + 3],
                ]
            } else {
                [
                    self.points[u + 3][v],
                    self.points[u + 3][v + 1],
                    self.points[u + 3][v + 2],
                    self.points[u + 3][v + 3],
                ]
            },
        ]
    }

    pub fn patch_edges(&self) -> Vec<C0Edge> {
        let u_patches = self.u_patches();
        let v_patches = self.v_patches();

        let mut edges = Vec::new();

        if !self.is_cylinder {
            for v in 0..v_patches {
                edges.push(C0Edge::new(self.patch(0, v)));
            }
        }

        for u in 0..u_patches {
            let patch = Self::rotate_patch(&Self::rotate_patch(&Self::rotate_patch(
                &self.patch(u, v_patches - 1),
            )));
            edges.push(C0Edge::new(patch));
        }

        if !self.is_cylinder {
            for v in 0..v_patches {
                let patch = Self::rotate_patch(&Self::rotate_patch(&self.patch(u_patches - 1, v)));
                edges.push(C0Edge::new(patch));
            }
        }

        for u in 0..u_patches {
            let patch = Self::rotate_patch(&self.patch(u, 0));
            edges.push(C0Edge::new(patch));
        }

        edges
    }

    fn gk_control(&mut self, ui: &imgui::Ui) {
        ui.checkbox("Wireframe", &mut self.wireframe);

        if self.wireframe {
            self.mesh.wireframe = true;
        } else {
            self.mesh.wireframe = false;
        }
    }

    fn draw_gk(&self, premul: &Matrix4<f32>, camera: &Camera) {
        let program = self.shader_manager.program("gk_mode");
        program.enable();
        program.uniform_matrix_4_f32_slice("model", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice("projection", camera.projection_transform().as_slice());

        self.mesh.draw();
    }
}

impl<'gl> ReferentialEntity<'gl> for BezierSurfaceC0<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("c0_surface_control");
        self.name_control_ui(ui);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);
        ui.checkbox("GK2 mode", &mut self.gk_mode);

        if self.gk_mode {
            self.gk_control(ui);
        } else {
            uv_subdivision_ui(ui, &mut self.u_patch_divisions, &mut self.v_patch_divisions);
        }

        ControlResult::default()
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &EntityCollection<'gl>,
    ) {
        self.recalculate_mesh(entities);
    }

    fn allow_deletion(&self, _deleted: &HashSet<usize>) -> bool {
        // Refuse deletion of any subscribed point
        false
    }

    fn notify_about_reindexing(
        &mut self,
        changes: &HashMap<usize, usize>,
        entities: &EntityCollection<'gl>,
    ) {
        for old_id in self.points.iter_mut().flatten() {
            if let Some(&new_id) = changes.get(old_id) {
                *old_id = new_id;
            }
        }

        self.recalculate_mesh(entities);
    }
}

impl<'gl> Drawable for BezierSurfaceC0<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        if self.gk_mode {
            self.draw_gk(premul, camera);
        } else {
            draw_bezier_surface(
                &self.mesh,
                self.u_patch_divisions,
                self.v_patch_divisions,
                &self.shader_manager,
                camera,
                premul,
                draw_type,
            );
        }

        if self.draw_bernstein_polygon {
            draw_polygon(
                &self.bernstein_polygon_mesh,
                &self.shader_manager,
                camera,
                premul,
                draw_type,
            );
        }
    }
}

impl<'gl> SceneObject for BezierSurfaceC0<'gl> {
    fn as_c0_surface(&self) -> Option<&BezierSurfaceC0> {
        Some(self)
    }

    fn as_parametric_2_to_3(&self) -> Option<Box<dyn DifferentialParametricForm<2, 3>>> {
        Some(Box::new(self.surface.clone()))
    }
}

impl<'gl> NamedEntity for BezierSurfaceC0<'gl> {
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
            "objectType": "bezierSurfaceC0",
            "name": self.name(),
            "patches": self.json_patches(),
            "parameterWrapped": {
                "u": self.is_cylinder,
                "v": false,
            },
            "size": {
                "x": self.u_patches(),
                "y": self.v_patches(),
            }
        })
    }
}
