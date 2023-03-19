use super::{
    basic::{Orientation, Scale, Translation},
    entity::{Entity, SceneObject},
};
use crate::{
    math::geometry::{self, gridable::Gridable},
    render::{drawable::Drawable, gl_program::GlProgram, mesh::LineMesh},
};
use nalgebra::{Matrix4, Point3};
use std::path::Path;

pub struct Torus<'gl> {
    torus: geometry::torus::Torus,
    mesh: LineMesh<'gl>,
    tube_points: u32,
    round_points: u32,
    orientation: Orientation,
    scale: Scale,
    translation: Translation,
    gl_program: GlProgram<'gl>,
}

impl<'gl> Torus<'gl> {
    pub fn new(gl: &'gl glow::Context) -> Torus<'gl> {
        let tube_points = 10;
        let round_points = 10;

        let torus = geometry::torus::Torus::with_radii(2.0, 0.5);
        let (vertices, topology) = torus.grid(round_points, tube_points);

        let mesh = LineMesh::new(gl, vertices, topology);

        let gl_program = GlProgram::with_shader_paths(
            gl,
            vec![
                (
                    Path::new("shaders/perspective_vertex.glsl"),
                    glow::VERTEX_SHADER,
                ),
                (
                    Path::new("shaders/simple_fragment.glsl"),
                    glow::FRAGMENT_SHADER,
                ),
            ],
        );

        Torus {
            torus,
            mesh,
            tube_points,
            round_points,
            gl_program,
            orientation: Orientation::new(),
            translation: Translation::new(),
            scale: Scale::new(),
        }
    }

    pub fn with_position(gl: &'gl glow::Context, position: Point3<f32>) -> Torus<'gl> {
        let mut torus = Torus::new(gl);
        torus.translation.translation = position.coords;
        torus
    }
}

macro_rules! safe_slider {
    ($ui:expr, $label:expr, $min:expr, $max:expr, $value:expr) => {
        $ui.slider_config($label, $min, $max)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build($value)
    };
}

impl<'gl> Entity for Torus<'gl> {
    fn control_ui(&mut self, ui: &imgui::Ui) {
        ui.text("Torus control");
        ui.separator();
        let mut torus_changed = false;
        torus_changed |= safe_slider!(ui, "R", 0.1, 10.0, &mut self.torus.inner_radius);
        torus_changed |= safe_slider!(ui, "r", 0.1, 10.0, &mut self.torus.tube_radius);
        torus_changed |= safe_slider!(ui, "M", 3, 50, &mut self.round_points);
        torus_changed |= safe_slider!(ui, "m", 3, 50, &mut self.tube_points);

        self.orientation.control_ui(ui);
        self.translation.control_ui(ui);
        self.scale.control_ui(ui);
        ui.separator();

        if torus_changed {
            let (vertices, indices) = self.torus.grid(self.round_points, self.tube_points);
            self.mesh.update_vertices(vertices, indices);
        }
    }
}

impl<'gl> SceneObject for Torus<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let model_transform =
            self.translation.as_matrix() * self.orientation.as_matrix() * self.scale.as_matrix();

        self.gl_program.enable();
        self.gl_program
            .uniform_matrix_4_f32_slice("model_transform", model_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
        self.gl_program
            .uniform_matrix_4_f32_slice("projection_transform", projection_transform.as_slice());
        self.mesh.draw();
    }
}
