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
    },
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

pub struct BezierSurfaceC0<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierSurfaceMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_bernstein_polygon: bool,

    points: Vec<Vec<usize>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    u_patch_divisions: u32,
    v_patch_divisions: u32,

    is_cyllinder: bool,
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

        Self {
            gl,
            mesh: BezierSurfaceMesh::new(gl, bezier_surface.clone()),
            points,
            bernstein_polygon_mesh: grid_mesh(gl, &bezier_surface.grid()),
            draw_bernstein_polygon: false,
            name: ChangeableName::new("Bezier Surface C0", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
            is_cyllinder: is_cylinder,
        }
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let bezier_surface = create_bezier_surface(&self.points, entities, self.is_cyllinder);
        self.mesh = BezierSurfaceMesh::new(self.gl, bezier_surface.clone());
        self.bernstein_polygon_mesh = grid_mesh(self.gl, &bezier_surface.grid());
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
        ui.checkbox("Draw De Boor polygon", &mut self.draw_bernstein_polygon);

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

impl<'gl> Drawable for BezierSurfaceC0<'gl> {
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

impl<'gl> SceneObject for BezierSurfaceC0<'gl> {}

impl<'gl> NamedEntity for BezierSurfaceC0<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }
}
