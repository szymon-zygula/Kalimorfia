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
        mill::{Cutter, CutterShape, Mill},
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
        generic_mesh::{CNCBlockVertex, GlMesh, Mesh},
        gl_drawable::GlDrawable,
        gl_texture::GlTexture,
        mesh::{LinesMesh, SurfaceVertex},
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use nalgebra::{vector, Matrix4, Vector2, Vector3};
use std::{cell::RefCell, rc::Rc, sync::mpsc};

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
    const MIN_SIZE: f32 = 100.0;
    const MIN_HEIGHT: f32 = 10.0;
    const MAX_SIZE: f32 = 400.0;
    const MIN_SAMPLING: i32 = 50;
    const MAX_SAMPLING: i32 = 4000;

    pub fn new() -> Self {
        Self {
            size: vector!(160.0, 160.0, 50.0),
            sampling: vector!(1000, 1000),
        }
    }

    pub fn clamp(&mut self) {
        self.size.x = self.size.x.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        self.size.y = self.size.y.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        self.size.z = self.size.z.clamp(Self::MIN_HEIGHT, Self::MAX_SIZE);

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

use std::time::Instant;

enum MeshMessage {
    #[allow(dead_code)]
    CreateNewMesh(Block),
    Exit,
}

pub struct CNCBlock<'gl> {
    gl: &'gl glow::Context,
    block: Option<Block>,
    mesh: GlMesh<'gl>,
    cutter_mesh: LinesMesh<'gl>,
    additional_mesh_translation: Matrix4<f32>,
    paths_mesh: LinesMesh<'gl>,
    draw_paths: bool,
    name: ChangeableName,
    shader_manager: Rc<ShaderManager<'gl>>,
    linear_transform: LinearTransformEntity,
    script_path: String,
    script_error: Option<String>,
    milling_player: Option<MillingPlayer>,
    playback_paused: bool,
    last_mesh_regen: Instant,
    mesh_regen_interval: f32,
    mesh_notifier: mpsc::Sender<MeshMessage>,
    mesh_receiver: mpsc::Receiver<Mesh<CNCBlockVertex>>,
    height_texture: GlTexture<'gl>,
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
        linear_transform.scale.scale = vector![0.05, 0.05, 0.05];
        linear_transform.orientation.axis = vector![1.0, 0.0, 0.0];
        linear_transform.orientation.angle =
            2.0 * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;

        let (mesh_sender, mesh_receiver) = std::sync::mpsc::channel::<Mesh<CNCBlockVertex>>();
        let (mesh_notifier, mesh_getter) = std::sync::mpsc::channel::<MeshMessage>();

        std::thread::spawn(move || {
            while let Ok(msg) = mesh_getter.recv() {
                let _ = match msg {
                    MeshMessage::CreateNewMesh(block) => mesh_sender.send(block.generate_mesh()),
                    MeshMessage::Exit => break,
                };
            }
        });

        Self {
            mesh: GlMesh::new(gl, &block.generate_mesh()),
            height_texture: GlTexture::new_float(
                gl,
                block.raw_heights(),
                block.sampling().x,
                block.sampling().y,
            ),
            cutter_mesh: LinesMesh::empty(gl),
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
            playback_paused: true,
            mesh_regen_interval: 0.0,
            last_mesh_regen: Instant::now(),
            mesh_notifier,
            mesh_receiver,
        }
    }

    pub fn request_new_mesh(&mut self) {
        let block = self
            .block
            .as_ref()
            .or(self
                .milling_player
                .as_ref()
                .map(|p| p.milling_process().block()))
            .unwrap();

        self.height_texture
            .load_float(block.raw_heights(), block.sampling().x, block.sampling().y);

        // if self.mesh_regen_interval == 0.0 {
        //     let mesh = block.generate_mesh();
        //     self.set_new_mesh(mesh);
        // } else {
        //     let _ = self
        //         .mesh_notifier
        //         .send(MeshMessage::CreateNewMesh(block.clone()));
        // }
    }

    pub fn try_receive_new_mesh(&mut self) {
        if let Ok(mesh) = self.mesh_receiver.try_recv() {
            self.set_new_mesh(mesh)
        }
    }

    pub fn set_new_mesh(&mut self, mesh: Mesh<CNCBlockVertex>) {
        self.mesh = GlMesh::new(self.gl, &mesh);
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
            let mut regen_mesh = false;

            if ui.button("Step") {
                player.full_step()?;
                regen_mesh = true;
            }

            if ui.button("Complete") {
                player.complete()?;
                regen_mesh = true;
            }

            if regen_mesh {
                self.request_new_mesh();
            }

            self.player_control(ui)?;
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

    fn merge_mesh(
        mesh_0: (Vec<SurfaceVertex>, Vec<u32>),
        mesh_1: (Vec<SurfaceVertex>, Vec<u32>),
    ) -> (Vec<SurfaceVertex>, Vec<u32>) {
        let mut points = mesh_0.0;
        let mut indices = mesh_0.1;
        let vertex_count_0 = points.len() as u32;
        indices.extend(mesh_1.1.into_iter().map(|u| u + vertex_count_0));

        points.extend(mesh_1.0);
        (points, indices)
    }

    fn create_new_cutter_mesh(&mut self, cutter: &Cutter) {
        let (mill_vertices, mill_indices) = match cutter {
            Cutter {
                shape: CutterShape::Cylinder,
                diameter,
                height,
            } => Cylinder::new(0.5 * *diameter as f64, *height as f64).grid(30, 30),
            Cutter {
                shape: CutterShape::Ball,
                diameter,
                height,
            } => {
                let sphere = Sphere::with_radius(0.5 * *diameter as f64).grid(30, 30);
                let cylinder =
                    Cylinder::new(0.5 * *diameter as f64, (height - 0.5 * diameter) as f64)
                        .grid(30, 30);

                let (v, i) = Self::merge_mesh(sphere, cylinder);
                (
                    v.into_iter()
                        .map(|v| SurfaceVertex {
                            point: v.point + vector![0.0, 0.0, 0.5 * diameter],
                            uv: v.uv,
                        })
                        .collect(),
                    i,
                )
            }
        };

        self.cutter_mesh = LinesMesh::new(
            self.gl,
            mill_vertices.iter().map(|v| v.point).collect(),
            mill_indices,
        );
    }

    fn use_program(&mut self, program: cncp::Program) {
        self.playback_paused = true;
        self.paths_mesh = LinesMesh::strip(self.gl, program.positions_sequence());
        self.create_new_cutter_mesh(&program.shape());

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

    fn player_control(&mut self, ui: &imgui::Ui) -> MillingResult {
        let Some(player) = &mut self.milling_player else {
            return Ok(());
        };

        let mut regen_mesh = false;

        if self.playback_paused {
            if ui.button("Play") {
                self.playback_paused = false;
                player.reset_timer();
            }
        } else {
            player.step()?;
            if ui.button("Pause") {
                self.playback_paused = true;
                regen_mesh = true;
            }
        }

        let cutter = &mut player.milling_process_mut().mill_mut().cutter;
        let mut regen_cutter = false;
        if ui
            .slider_config("Cutter height", cutter.diameter, 100.0)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build(&mut cutter.height)
        {
            regen_cutter = true;
        }

        if ui
            .slider_config("Cutter diameter", 0.5, 20.0)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build(&mut cutter.diameter)
        {
            regen_cutter = true;
        }

        let mut cylinder = matches!(
            player.milling_process().mill().cutter.shape,
            CutterShape::Cylinder
        );

        if ui.checkbox("Cylinder cutter", &mut cylinder) {
            player.milling_process_mut().mill_mut().cutter.shape = if cylinder {
                CutterShape::Cylinder
            } else {
                CutterShape::Ball
            };

            regen_cutter = true;
        }

        ui.slider_config(
            "Block base height",
            1.0,
            player.milling_process().block().block_height(),
        )
        .flags(imgui::SliderFlags::NO_INPUT)
        .build(&mut player.milling_process_mut().block_mut().base_height);

        ui.slider_config("Simulation speed", 1.0, 1000.0)
            .flags(imgui::SliderFlags::LOGARITHMIC | imgui::SliderFlags::NO_INPUT)
            .build(&mut player.slow_speed);

        ui.slider_config("Mesh regeneration interval", 0.0, 1.0)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build(&mut self.mesh_regen_interval);

        if !self.playback_paused
            && (Instant::now() - self.last_mesh_regen).as_secs_f32() >= self.mesh_regen_interval
        {
            regen_mesh = true;
            self.last_mesh_regen = Instant::now();
        }

        if regen_cutter {
            let cutter = player.milling_process().mill().cutter;
            self.create_new_cutter_mesh(&cutter);
        }

        if regen_mesh {
            self.request_new_mesh();
        }

        Ok(())
    }
}

impl<'gl> Entity for CNCBlock<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);

        if self.script_error.is_none() {
            if let Err(err) = self.milling_control(ui) {
                self.playback_paused = true;
                self.request_new_mesh();
                self.script_error = Some(err.to_string());
            }
        }

        self.try_receive_new_mesh();

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
        self.height_texture.bind();

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
            program.uniform_color("vertex_color", &Color::red());
            self.cutter_mesh.draw();

            if self.draw_paths {
                program.uniform_color("vertex_color", &Color::green());
                program.uniform_matrix_4_f32_slice(
                    "model_transform",
                    (premul
                        * model_transform
                        // Avoid z-fighting with CNC mesh
                        * transforms::translate(vector![0.0, 0.0, 0.01])
                        * self.additional_mesh_translation)
                        .as_slice(),
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

impl<'gl> Drop for CNCBlock<'gl> {
    fn drop(&mut self) {
        let _ = self.mesh_notifier.send(MeshMessage::Exit);
    }
}
