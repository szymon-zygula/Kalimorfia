use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject},
};
use crate::cnc::block::Block;
use crate::{
    camera::Camera,
    render::{
        generic_mesh::GlMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use nalgebra::{vector, Matrix4, Vector2, Vector3};
use std::{cell::RefCell, rc::Rc};

pub struct CNCBlockArgs {
    pub size: Vector3<f32>,
    pub sampling: Vector2<i32>,
}

impl Default for CNCBlockArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl CNCBlockArgs {
    const MIN_SIZE: f32 = 1.0;
    const MAX_SIZE: f32 = 10.0;
    const MIN_SAMPLING: i32 = 50;
    const MAX_SAMPLING: i32 = 300;

    pub fn new() -> Self {
        Self {
            size: vector!(5.0, 5.0, 2.5),
            sampling: vector!(100, 100),
        }
    }

    pub fn clamp(&mut self) {
        self.size.x = self.size.x.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        self.size.y = self.size.y.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        self.size.z = self.size.z.clamp(Self::MIN_SIZE, Self::MAX_SIZE);

        self.sampling.x = self
            .sampling
            .x
            .clamp(Self::MIN_SAMPLING, Self::MAX_SAMPLING);
        self.sampling.y = self
            .sampling
            .y
            .clamp(Self::MIN_SAMPLING, Self::MAX_SAMPLING);
    }
}

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
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        args: CNCBlockArgs,
    ) -> Self {
        let block = Block::new(
            vector!(args.sampling.x as usize, args.sampling.y as usize),
            args.size,
        );

        let mut linear_transform = LinearTransformEntity::new();
        linear_transform.orientation.axis = vector![1.0, 0.0, 0.0];
        linear_transform.orientation.angle =
            2.0 * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;

        Self {
            mesh: block.generate_mesh(gl),
            gl,
            block,
            shader_manager,
            linear_transform,
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
        self.linear_transform.control_ui(ui);
        false
    }
}

impl<'gl> Drawable for CNCBlock<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, _: DrawType) {
        let model_transform = self.linear_transform.matrix();

        let program = self.shader_manager.program("cnc_block");
        program.enable();
        program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_3_f32(
            "cam_pos",
            camera.position().x,
            camera.position().y,
            camera.position().z,
        );

        self.mesh.draw();
    }
}

impl<'gl> SceneObject for CNCBlock<'gl> {
    fn model_transform(&self) -> Matrix4<f32> {
        self.linear_transform.matrix()
    }

    fn set_model_transform(&mut self, linear_transform: LinearTransformEntity) {
        self.linear_transform = linear_transform;
    }
}

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
