use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject},
};
use crate::{
    camera::Camera,
    cnc::program as cncp,
    cnc::{
        block::Block,
        mill::{Mill, MillShape, MillType},
        milling_player::MillingPlayer,
        milling_process::MillingProcess,
        milling_process::MillingResult,
    },
    math::{
        affine::transforms,
        geometry::{cylinder::Cylinder, gridable::Gridable, sphere::Sphere},
    },
    primitives::color::Color,
    render::{
        generic_mesh::GlMesh,
        gl_drawable::GlDrawable,
        mesh::{LinesMesh, SurfaceVertex},
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
    const MIN_SIZE: f32 = 10.0;
    const MAX_SIZE: f32 = 400.0;
    const MIN_SAMPLING: i32 = 50;
    const MAX_SAMPLING: i32 = 1000;

    pub fn new() -> Self {
        Self {
            size: vector!(250.0, 250.0, 50.0),
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
    block: Option<Block>,
    mesh: GlMesh<'gl>,
    mill_mesh: LinesMesh<'gl>,
    additional_mesh_translation: Matrix4<f32>,
    paths_mesh: LinesMesh<'gl>,
    draw_paths: bool,
    name: ChangeableName,
    shader_manager: Rc<ShaderManager<'gl>>,
    linear_transform: LinearTransformEntity,
    script_path: String,
    script_error: Option<String>,
    milling_player: Option<MillingPlayer>,
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
        linear_transform.scale.scale = vector![0.01, 0.01, 0.01];
        linear_transform.orientation.axis = vector![1.0, 0.0, 0.0];
        linear_transform.orientation.angle =
            2.0 * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;

        Self {
            mesh: block.generate_mesh(gl),
            mill_mesh: LinesMesh::empty(gl),
            additional_mesh_translation: transforms::translate(vector![
                block.size().x * 0.5,
                block.size().y * 0.5,
                0.0
            ]),
            draw_paths: true,
            paths_mesh: LinesMesh::empty(gl),
            gl,
            block: Some(block),
            shader_manager,
            linear_transform,
            name: ChangeableName::new("CNC block", name_repo),
            script_path: String::from("paths/1.k16"),
            script_error: None,
            milling_player: None,
        }
    }

    pub fn block_mut(&mut self) -> Option<&mut Block> {
        self.block.as_mut()
    }

    pub fn block(&self) -> Option<&Block> {
        self.block.as_ref()
    }

    fn milling_control(&mut self, ui: &imgui::Ui) -> MillingResult {
        ui.text("Milling control");
        self.load_script_ui(ui);

        if let Some(player) = &mut self.milling_player {
            ui.text("Milling player");
            ui.text(format!(
                "Executed: {}/{}",
                player.milling_process().current_instruction_idx(),
                player.milling_process().program().instructions().len()
            ));
            let position = player.milling_process().mill().position();
            ui.text(format!(
                "Mill position: [{}, {}, {}]",
                position.x, position.y, position.z,
            ));

            ui.checkbox("Draw paths", &mut self.draw_paths);

            if ui.button("Step") {
                player.full_step()?;
                // TODO: could be more optimal
                self.mesh = player.milling_process().block().generate_mesh(self.gl);
            }

            if ui.button("Complete") {
                player.complete()?;
                self.mesh = player.milling_process().block().generate_mesh(self.gl);
            }
        }

        Ok(())
    }

    fn load_script_ui(&mut self, ui: &imgui::Ui) {
        if ui.button("Load script") {
            ui.open_popup("mill_path_popup");
        }

        ui.popup("mill_path_popup", || {
            ui.input_text("File path", &mut self.script_path).build();
            if ui.button("Open") {
                let program =
                    cncp::Program::from_file(std::path::Path::new(&self.script_path), true);
                match program {
                    Err(err) => {
                        self.script_error = Some(err.to_string());
                    }
                    Ok(prog) => {
                        self.use_program(prog);
                    }
                }

                ui.close_current_popup();
            }
        });
    }

    fn use_program(&mut self, program: cncp::Program) {
        self.paths_mesh = LinesMesh::strip(self.gl, program.positions_sequence());
        let (mill_vertices, mill_indices) = match program.shape() {
            MillShape {
                type_: MillType::Cylinder,
                diameter,
            } => Cylinder::new(0.5 * diameter as f64, 3.0 * diameter as f64).grid(30, 30),
            MillShape {
                type_: MillType::Ball,
                diameter,
            } => {
                let (v, i) = Sphere::with_radius(0.5 * diameter as f64).grid(30, 30);
                (
                    v.iter()
                        .map(|v| SurfaceVertex {
                            point: v.point + vector![0.0, 0.0, 0.5 * diameter],
                            uv: v.uv,
                        })
                        .collect(),
                    i,
                )
            }
        };
        self.mill_mesh = LinesMesh::new(
            self.gl,
            mill_vertices.iter().map(|v| v.point).collect(),
            mill_indices,
        );

        if let Some(player) = self.milling_player.take() {
            self.block = Some(player.take().retake_all().2);
        }

        let mut mill = Mill::new(program.shape());
        mill.move_to(vector![
            0.0,
            0.0,
            2.0 * self.block.as_ref().unwrap().block_height()
        ])
        .unwrap();

        let process = MillingProcess::new(mill, program, self.block.take().unwrap());
        self.milling_player = Some(MillingPlayer::new(process));
    }
}

impl<'gl> Entity for CNCBlock<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);

        if let Err(err) = self.milling_control(ui) {
            self.script_error = Some(err.to_string());
        }

        let err = self.script_error.clone();
        if let Some(err) = err {
            ui.window("Milling error")
                .size([400.0, 100.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text_colored([1.0, 0.3, 0.3, 1.0], format!("Error: {}", err));
                    if ui.button("OK") {
                        self.script_error = None;
                        ui.close_current_popup();
                    }
                });
        }

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

        if let Some(player) = &self.milling_player {
            let program = self.shader_manager.program("spline");
            program.enable();
            program.uniform_matrix_4_f32_slice(
                "model_transform",
                (premul
                    * model_transform
                    * self.additional_mesh_translation
                    * transforms::translate(*player.milling_process().mill().position()))
                .as_slice(),
            );
            program
                .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
            program.uniform_matrix_4_f32_slice(
                "projection_transform",
                camera.projection_transform().as_slice(),
            );
            program.uniform_color("vertex_color", &Color::blue());
            self.mill_mesh.draw();

            if self.draw_paths {
                program.uniform_color("vertex_color", &Color::green());
                program.uniform_matrix_4_f32_slice(
                    "model_transform",
                    (premul * model_transform * self.additional_mesh_translation).as_slice(),
                );
                self.paths_mesh.draw();
            }
        }
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
