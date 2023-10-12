use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{
        ControlResult, DrawType, Drawable, Entity, EntityCollection, NamedEntity, SceneObject,
    },
    utils,
};
use crate::cnc::block::Block;
use crate::{
    camera::Camera,
    math::geometry,
    primitives::color::Color,
    render::{
        bezier_mesh::BezierMesh, generic_mesh::GlMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
    ui::ordered_selector,
};
use nalgebra::Matrix4;
use std::{cell::RefCell, rc::Rc};

pub struct CNCBlock<'gl> {
    gl: &'gl glow::Context,
    block: Block,
    mesh: GlMesh<'gl>,
    name: ChangeableName,
    shader_manager: Rc<ShaderManager<'gl>>,
    linear_transform: LinearTransformEntity,
}

impl<'gl> CNCBlock<'gl> {
    pub fn new(
        block: Block,
        gl: &glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Self {
        Self {
            mesh: block.generate_mesh(gl),
            block,
            gl,
            shader_manager,
            linear_transform: LinearTransformEntity::new(),
            name: ChangeableName::new("CNC block", name_repo),
        }
    }

    pub fn block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn block(&self) -> &Block {
        &self.block
    }
}

impl<'gl> Entity for CNCBlock<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);
        false
    }
}

impl<'gl> Drawable for CNCBlock<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let model_transform = self.linear_transform.matrix();

        let program = self.shader_manager.program("point");
        program.enable();
        program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        self.mesh.draw();
    }
}

impl<'gl> SceneObject for CNCBlock<'gl> {}

impl<'gl> NamedEntity for CNCBlock<'gl> {
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
            "objectType": "cncBlock",
            "name": self.name()
        })
    }
}
