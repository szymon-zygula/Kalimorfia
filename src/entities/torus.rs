use super::{
    basic::LinearTransformEntity,
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject,},
};
use crate::{
    camera::Camera,
    math::{
        decompositions::tait_bryan::TaitBryanDecomposition,
        geometry::{self, gridable::Gridable},
    },
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, mesh::LinesMesh, shader_manager::ShaderManager},
    repositories::NameRepository,
};
use nalgebra::{Matrix4, Point3};
use std::{cell::RefCell, rc::Rc};

pub struct Torus<'gl> {
    torus: geometry::torus::Torus,
    mesh: LinesMesh<'gl>,
    tube_points: u32,
    round_points: u32,
    linear_transform: LinearTransformEntity,
    name: ChangeableName,
    shader_manager: Rc<ShaderManager<'gl>>,
}

impl<'gl> Torus<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Torus<'gl> {
        let tube_points = 10;
        let round_points = 10;

        let torus = geometry::torus::Torus::with_radii(2.0, 0.5);
        let (vertices, topology) = torus.grid(round_points, tube_points);

        let mesh = LinesMesh::new(gl, vertices, topology);

        Torus {
            torus,
            mesh,
            tube_points,
            round_points,
            shader_manager,
            linear_transform: LinearTransformEntity::new(),
            name: ChangeableName::new("Torus", name_repo),
        }
    }

    pub fn with_position(
        gl: &'gl glow::Context,
        position: Point3<f32>,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
    ) -> Torus<'gl> {
        let mut torus = Torus::new(gl, name_repo, shader_manager);
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
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let model_transform = self.model_transform();

        let program = self.shader_manager.program("torus");
        program.enable();
        program
            .uniform_matrix_4_f32_slice("model_transform", (premul * model_transform).as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));
        self.mesh.draw();
    }
}

impl<'gl> SceneObject for Torus<'gl> {
    fn location(&self) -> Option<Point3<f32>> {
        Some(self.linear_transform.translation.translation.into())
    }

    fn model_transform(&self) -> Matrix4<f32> {
        self.linear_transform.matrix()
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

    fn to_json(&self) -> serde_json::Value {
        let decomposition =
            TaitBryanDecomposition::decompose(&self.linear_transform.orientation.matrix());
        serde_json::json!({
            "objectType": "torus",
            "position": {
                "x": self.linear_transform.translation.translation.x,
                "y": self.linear_transform.translation.translation.y,
                "z": self.linear_transform.translation.translation.z
            },
            "rotation": {
                "x": decomposition.x,
                "y": decomposition.y,
                "z": decomposition.z
            },
            "scale": {
                "x": self.linear_transform.scale.scale.x,
                "y": self.linear_transform.scale.scale.y,
                "z": self.linear_transform.scale.scale.z
            },
            "samples": {
                "x": self.round_points,
                "y": self.tube_points
            },
            "smallRadius": self.torus.tube_radius,
            "largeRadius": self.torus.inner_radius
        })
    }
}
