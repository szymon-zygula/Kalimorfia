use super::{
    basic::Translation,
    entity::{Entity, SceneObject},
    screen_coordinates::ScreenCoordinates,
};
use crate::{
    camera::Camera,
    math::affine::transforms,
    primitives::vertex::ColoredVertex,
    render::{drawable::Drawable, gl_program::GlProgram, mesh::ColoredLineMesh},
};
use nalgebra::{Matrix4, Point3};
use std::path::Path;

pub struct Cursor<'gl> {
    position: Translation,
    mesh: ColoredLineMesh<'gl>,
    gl_program: GlProgram<'gl>,
    scale: f32,
}

impl<'gl> Cursor<'gl> {
    pub fn new(gl: &glow::Context, scale: f32) -> Cursor {
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

        let gl_program = GlProgram::with_shader_paths(
            gl,
            vec![
                (
                    Path::new("shaders/perspective_vertex_colored.glsl"),
                    glow::VERTEX_SHADER,
                ),
                (
                    Path::new("shaders/fragment_colored.glsl"),
                    glow::FRAGMENT_SHADER,
                ),
            ],
        );

        Cursor {
            position: Translation::new(),
            mesh,
            gl_program,
            scale,
        }
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position.translation = position.coords;
    }
}

impl<'gl> Entity for Cursor<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        let _token = ui.push_id("cursor");
        ui.text("Cursor control");
        self.position.control_ui(ui)
    }
}

impl<'gl> SceneObject for Cursor<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let model_transform = self.position.as_matrix() * transforms::uniform_scale(self.scale);

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", model_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("projection_transform", projection_transform.as_slice());
        self.mesh.draw();
    }

    fn location(&self) -> Point3<f32> {
        self.position.translation.into()
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
        resolution: glutin::dpi::PhysicalSize<u32>,
    ) -> ScreenCursor<'gl> {
        ScreenCursor {
            cursor: Cursor::new(gl, 1.0),
            screen_coordinates: ScreenCoordinates::new(resolution),
            camera,
        }
    }

    fn update_coords_from_world(&mut self) {
        let screen_coords = Point3::from_homogeneous(
            self.camera.projection_transform()
                * self.camera.view_transform()
                * Point3::from(self.cursor.position.translation).to_homogeneous(),
        )
        .unwrap();

        self.screen_coordinates
            .set_ndc_coords(screen_coords.xy());
    }

    fn update_world_from_coords(&mut self) {}

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

impl<'gl> SceneObject for ScreenCursor<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        self.cursor.draw(projection_transform, view_transform)
    }

    fn location(&self) -> Point3<f32> {
        self.cursor.location()
    }
}
