use super::{
    basic::Translation,
    entity::{DrawType, Drawable, Entity, SceneObject},
    screen_coordinates::ScreenCoordinates,
};
use crate::{
    camera::Camera,
    math::affine::transforms,
    primitives::vertex::ColoredVertex,
    render::{gl_drawable::GlDrawable, mesh::ColoredLineMesh, shader_manager::ShaderManager},
};
use nalgebra::{Matrix4, Point3};
use std::rc::Rc;

pub struct Cursor<'gl> {
    position: Option<Translation>,
    mesh: ColoredLineMesh<'gl>,
    shader_manager: Rc<ShaderManager<'gl>>,
    scale: f32,
}

impl<'gl> Cursor<'gl> {
    pub fn new(gl: &'gl glow::Context, shader_manager: Rc<ShaderManager<'gl>>, scale: f32) -> Self {
        let mut mesh = ColoredLineMesh::new(
            gl,
            vec![
                ColoredVertex::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                ColoredVertex::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
                ColoredVertex::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
                ColoredVertex::new(0.0, 1.0, 0.0, 0.0, 1.0, 0.0),
                ColoredVertex::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
                ColoredVertex::new(0.0, 0.0, 1.0, 0.0, 0.0, 1.0),
            ],
            vec![0, 1, 2, 3, 4, 5],
        );

        mesh.as_line_mesh_mut().thickness(3.0);

        Self {
            position: Some(Translation::new()),
            mesh,
            shader_manager,
            scale,
        }
    }

    pub fn set_position(&mut self, position: Option<Point3<f32>>) {
        self.position = position.map(|p| Translation::with(p.coords));
    }
}

impl<'gl> Entity for Cursor<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("cursor");
        ui.text("Cursor control");

        if let Some(ref mut position) = self.position {
            position.control_ui(ui)
        } else {
            false
        }
    }
}

impl<'gl> Drawable for Cursor<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, _draw_type: DrawType) {
        if let Some(ref position) = self.position {
            let model_transform = position.matrix() * transforms::uniform_scale(self.scale);

            let program = self.shader_manager.program("cursor");
            program.enable();
            program.uniform_matrix_4_f32_slice(
                "model_transform",
                (premul * model_transform).as_slice(),
            );
            program
                .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
            program.uniform_matrix_4_f32_slice(
                "projection_transform",
                camera.projection_transform().as_slice(),
            );
            self.mesh.draw();
        }
    }
}

impl<'gl> SceneObject for Cursor<'gl> {
    fn location(&self) -> Option<Point3<f32>> {
        self.position.as_ref().map(|p| p.translation.into())
    }
}

pub struct ScreenCursor<'gl> {
    cursor: Cursor<'gl>,
    screen_coordinates: ScreenCoordinates,
    camera: Camera,
}

impl<'gl> ScreenCursor<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        camera: Camera,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Self {
        Self {
            cursor: Cursor::new(gl, shader_manager, 1.0),
            screen_coordinates: ScreenCoordinates::new(camera.resolution),
            camera,
        }
    }

    fn ndc_from_world(&self) -> Point3<f32> {
        self.camera.world_to_ndc(&self.world_position())
    }

    fn update_coords_from_world(&mut self) {
        let ndc_coords = self.ndc_from_world();
        self.screen_coordinates.set_ndc(ndc_coords.xy());
    }

    fn update_world_from_coords(&mut self) {
        let old_world = self.world_position();
        let ndc = self.screen_coordinates.get_ndc();

        self.cursor.position.as_mut().unwrap().translation =
            self.camera.move_world_to_ndc(&old_world, &ndc).coords;
    }

    fn world_position(&self) -> Point3<f32> {
        Point3::from(self.cursor.position.as_ref().unwrap().translation)
    }

    pub fn set_camera(&mut self, camera: &Camera) {
        self.camera = camera.clone();
        self.update_coords_from_world();
    }

    pub fn set_camera_and_resolution(
        &mut self,
        camera: &Camera,
        resolution: &glutin::dpi::PhysicalSize<u32>,
    ) {
        self.camera = camera.clone();
        self.screen_coordinates.set_resolution(*resolution);
        self.update_coords_from_world();
    }
}

impl<'gl> Entity for ScreenCursor<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("screen_cursor");
        let mut changed = if self.cursor.control_ui(ui) {
            self.update_coords_from_world();
            true
        } else {
            false
        };

        changed |= if self.screen_coordinates.control_ui(ui) {
            self.update_world_from_coords();
            true
        } else {
            false
        };

        changed
    }
}

impl<'gl> Drawable for ScreenCursor<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        self.cursor.draw(camera, premul, draw_type)
    }
}

impl<'gl> SceneObject for ScreenCursor<'gl> {
    fn location(&self) -> Option<Point3<f32>> {
        self.cursor.location()
    }
}
