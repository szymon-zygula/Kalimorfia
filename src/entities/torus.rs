use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{Drawable, Entity, NamedEntity, SceneObject},
};
use crate::{
    math::geometry::{self, gridable::Gridable},
    render::{gl_drawable::GlDrawable, gl_program::GlProgram, mesh::LinesMesh},
    repositories::NameRepository,
};
use nalgebra::{Matrix4, Point3};
use std::{cell::RefCell, path::Path, rc::Rc};

pub struct Torus<'gl> {
    torus: geometry::torus::Torus,
    mesh: LinesMesh<'gl>,
    tube_points: u32,
    round_points: u32,
    linear_transform: LinearTransformEntity,
    gl_program: GlProgram<'gl>,
    name: ChangeableName,
}

impl<'gl> Torus<'gl> {
    pub fn new(gl: &'gl glow::Context, name_repo: Rc<RefCell<dyn NameRepository>>) -> Torus<'gl> {
        let tube_points = 10;
        let round_points = 10;

        let torus = geometry::torus::Torus::with_radii(2.0, 0.5);
        let (vertices, topology) = torus.grid(round_points, tube_points);

        let mesh = LinesMesh::new(gl, vertices, topology);

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
            linear_transform: LinearTransformEntity::new(),
            name: ChangeableName::new("Torus", name_repo),
        }
    }

    pub fn with_position(
        gl: &'gl glow::Context,
        position: Point3<f32>,
        name_repo: Rc<RefCell<dyn NameRepository>>,
    ) -> Torus<'gl> {
        let mut torus = Torus::new(gl, name_repo);
        torus.linear_transform.translation.translation = position.coords;
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
    fn control_ui(&mut self, ui: &imgui::Ui) -> bool {
        self.name_control_ui(ui);
        let mut torus_changed = false;
        torus_changed |= safe_slider!(ui, "R", 0.1, 10.0, &mut self.torus.inner_radius);
        torus_changed |= safe_slider!(ui, "r", 0.1, 10.0, &mut self.torus.tube_radius);
        torus_changed |= safe_slider!(ui, "M", 3, 50, &mut self.round_points);
        torus_changed |= safe_slider!(ui, "m", 3, 50, &mut self.tube_points);

        self.linear_transform.control_ui(ui);
        ui.separator();

        if torus_changed {
            let (vertices, indices) = self.torus.grid(self.round_points, self.tube_points);
            self.mesh.update_vertices(vertices, indices);
        }

        torus_changed
    }
}

impl<'gl> Drawable for Torus<'gl> {
    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let model_transform = self.model_transform();

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

impl<'gl> SceneObject for Torus<'gl> {
    fn location(&self) -> Option<Point3<f32>> {
        Some(self.linear_transform.translation.translation.into())
    }

    fn model_transform(&self) -> Matrix4<f32> {
        self.linear_transform.as_matrix()
    }

    fn set_model_transform(&mut self, linear_transform: LinearTransformEntity) {
        self.linear_transform = linear_transform;
    }
}

impl<'gl> NamedEntity for Torus<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }
}
