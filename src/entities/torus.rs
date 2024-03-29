use super::{
    basic::{IntersectionTexture, LinearTransformEntity},
    changeable_name::ChangeableName,
    entity::{DrawType, Drawable, Entity, NamedEntity, SceneObject},
};
use crate::{
    camera::Camera,
    math::{
        decompositions::tait_bryan::TaitBryanDecomposition,
        geometry::{self, gridable::Gridable, parametric_form::DifferentialParametricForm},
        utils::mat_32_to_64,
    },
    primitives::color::Color,
    render::{
        gl_drawable::GlDrawable, mesh::TorusMesh, shader_manager::ShaderManager, texture::Texture,
    },
    repositories::NameRepository,
};
use nalgebra::{Matrix4, Point3};
use std::{cell::RefCell, rc::Rc};

pub struct Torus<'gl> {
    gl: &'gl glow::Context,
    pub torus: geometry::torus::Torus,
    mesh: TorusMesh<'gl>,
    pub tube_points: u32,
    pub round_points: u32,
    pub linear_transform: LinearTransformEntity,
    pub name: ChangeableName,
    intersection_texture: IntersectionTexture<'gl>,
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

        let mesh = TorusMesh::new(gl, vertices, topology);

        Torus {
            gl,
            torus,
            mesh,
            tube_points,
            round_points,
            shader_manager,
            linear_transform: LinearTransformEntity::new(),
            name: ChangeableName::new("Torus", name_repo),
            intersection_texture: IntersectionTexture::empty(gl, true, true),
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

    pub fn regenerate_mesh(&mut self) {
        let (vertices, indices) = self.torus.grid(self.round_points, self.tube_points);
        self.mesh.update_vertices(vertices, indices);
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
        let _token = ui.push_id(self.name());
        self.name_control_ui(ui);
        let mut torus_changed = false;
        torus_changed |= safe_slider!(ui, "R", 0.1, 10.0, &mut self.torus.inner_radius);
        torus_changed |= safe_slider!(ui, "r", 0.1, 10.0, &mut self.torus.tube_radius);
        torus_changed |= safe_slider!(ui, "M", 3, 50, &mut self.round_points);
        torus_changed |= safe_slider!(ui, "m", 3, 50, &mut self.tube_points);

        self.linear_transform.control_ui(ui);
        ui.separator();

        self.intersection_texture.control_ui(ui);

        if torus_changed {
            self.regenerate_mesh();
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
        program.uniform_color("color", &Color::for_draw_type(&draw_type));
        self.intersection_texture.bind();
        self.mesh.draw();
    }
}

impl<'gl> SceneObject for Torus<'gl> {
    fn location(&self) -> Option<Point3<f32>> {
        Some(self.linear_transform.translation.translation.into())
    }

    fn as_parametric_2_to_3(
        &self,
    ) -> Option<Box<dyn DifferentialParametricForm<2, 3> + Send + Sync>> {
        Some(Box::new(geometry::torus::AffineTorus::new(
            self.torus,
            mat_32_to_64(self.linear_transform.matrix()),
        )))
    }

    fn set_intersection_texture(&mut self, texture: Texture) {
        self.intersection_texture = IntersectionTexture::new(self.gl, texture, true, true);
    }

    fn intersection_texture(&self) -> Option<&IntersectionTexture<'gl>> {
        Some(&self.intersection_texture)
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

    fn set_similar_name(&mut self, name: &str) {
        self.name.set_similar_name(name)
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
                "x": decomposition.x.to_degrees(),
                "y": decomposition.y.to_degrees(),
                "z": decomposition.z.to_degrees()
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
            "largeRadius": self.torus.inner_radius,
            "name": self.name()
        })
    }
}
