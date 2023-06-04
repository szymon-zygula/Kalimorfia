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

pub struct GregoryPatch<'gl> {
    gl: &'gl glow::Context,

    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    pub u_patch_divisions: u32,
    pub v_patch_divisions: u32,
}

impl<'gl> GregoryPatch<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        entities: &EntityCollection<'gl>,
    ) -> Self {
        Self {
            gl,
            name: ChangeableName::new("Gregory patch", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
        }
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        todo!()
    }
}

impl<'gl> ReferentialEntity<'gl> for GregoryPatch<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("gregory_control");
        self.name_control_ui(ui);

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
        // Refuse deletion of any subscribed points or surfaces
        false
    }

    fn notify_about_reindexing(
        &mut self,
        changes: &HashMap<usize, usize>,
        entities: &EntityCollection<'gl>,
    ) {
        todo!();

        self.recalculate_mesh(entities);
    }
}

impl<'gl> Drawable for GregoryPatch<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        todo!()
    }
}

impl<'gl> SceneObject for GregoryPatch<'gl> {}

impl<'gl> NamedEntity for GregoryPatch<'gl> {
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
            "objectType": "gregoryPatch",
            "name": self.name(),
        })
    }
}
