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
    math::geometry::bezier::{deboor_surface_to_bernstein, BezierSurface},
    render::{
        bezier_surface_mesh::BezierSurfaceMesh, mesh::LinesMesh, shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use nalgebra::Matrix4;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct BezierSurfaceC2<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierSurfaceMesh<'gl>,
    deboor_polygon_mesh: LinesMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_deboor_polygon: bool,
    draw_bernstein_polygon: bool,

    points: Vec<Vec<usize>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    pub u_patch_divisions: u32,
    pub v_patch_divisions: u32,

    is_cylinder: bool,
}

impl<'gl> BezierSurfaceC2<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        points: Vec<Vec<usize>>,
        entities: &EntityCollection<'gl>,
        args: BezierSurfaceArgs,
    ) -> Self {
        let is_cylinder = matches!(args, BezierSurfaceArgs::Cylinder(..));
        let mut s = Self {
            gl,
            mesh: BezierSurfaceMesh::empty(gl),
            deboor_polygon_mesh: LinesMesh::empty(gl),
            bernstein_polygon_mesh: LinesMesh::empty(gl),
            points,
            draw_deboor_polygon: false,
            draw_bernstein_polygon: false,
            name: ChangeableName::new("Bezier Surface C0", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
            is_cylinder,
        };

        s.recalculate_mesh(entities);

        s
    }

    pub fn wrapped_points(&self) -> Vec<Vec<usize>> {
        let mut points = self.points.clone();

        if self.is_cylinder {
            points.push(points[0].clone());
            points.push(points[1].clone());
            points.push(points[2].clone());
        }

        points
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let wrapped_points = self.wrapped_points();
        let deboor_points = point_ids_to_f64(&wrapped_points, entities);
        let bernstein_points = deboor_surface_to_bernstein(deboor_points);
        let bezier_surface = BezierSurface::new(bernstein_points);

        self.mesh = BezierSurfaceMesh::new(self.gl, bezier_surface.clone());
        self.bernstein_polygon_mesh = grid_mesh(self.gl, bezier_surface.grid());

        let deboor_grid = create_grid(&self.points, entities, self.is_cylinder);
        self.deboor_polygon_mesh = grid_mesh(self.gl, &deboor_grid);
    }

    fn u_patches(&self) -> usize {
        if self.is_cylinder {
            self.points.len()
        } else {
            self.points.len() - 3
        }
    }

    fn v_patches(&self) -> usize {
        self.points.first().map_or(0, |first| first.len() - 3)
    }

    fn patch_control_points(&self, patch_u: usize, patch_v: usize) -> Vec<usize> {
        let mut points = Vec::new();

        for v in 0..4 {
            for u in 0..4 {
                points.push(self.points[(patch_u + u) % self.points.len()][patch_v + v]);
            }
        }

        points
    }

    fn json_patches(&self) -> Vec<serde_json::Value> {
        let mut patches = Vec::new();

        let u_patches = self.u_patches();
        let v_patches = self.v_patches();

        for patch_u in 0..u_patches {
            for patch_v in 0..v_patches {
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
}

impl<'gl> ReferentialEntity<'gl> for BezierSurfaceC2<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("c2_surface_control");
        self.name_control_ui(ui);
        ui.checkbox("Draw de Boor polygon", &mut self.draw_deboor_polygon);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);

        uv_subdivision_ui(ui, &mut self.u_patch_divisions, &mut self.v_patch_divisions);

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
}

impl<'gl> Drawable for BezierSurfaceC2<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        draw_bezier_surface(
            &self.mesh,
            self.u_patch_divisions,
            self.v_patch_divisions,
            &self.shader_manager,
            camera,
            premul,
            draw_type,
        );

        if self.draw_deboor_polygon {
            draw_polygon(
                &self.deboor_polygon_mesh,
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

impl<'gl> SceneObject for BezierSurfaceC2<'gl> {}

impl<'gl> NamedEntity for BezierSurfaceC2<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "objectType": "bezierSurfaceC2",
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
