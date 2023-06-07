use crate::{
    camera::Camera,
    entities::{
        bezier_utils::*,
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, Drawable, EntityCollection, NamedEntity, ReferentialEntity,
            SceneObject,
        },
    },
    graph::{C0Edge, C0EdgeTriangle},
    math::geometry::gregory::{BorderPatch, GregoryTriangle},
    primitives::{color::Color, vertex::ColoredVertex},
    render::{
        bezier_surface_mesh::GregoryMesh, gl_drawable::GlDrawable, mesh::ColoredLineMesh,
        point_cloud::PointCloud, shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use glow::HasContext;
use nalgebra::{Matrix4, Point3, Vector3};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct GregoryPatch<'gl> {
    gl: &'gl glow::Context,

    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    pub u_patch_divisions: u32,
    pub v_patch_divisions: u32,

    triangle: C0EdgeTriangle,
    mesh: GregoryMesh<'gl>,
    vector_meshes: Vec<ColoredLineMesh<'gl>>,
    control_points_meshes: [PointCloud<'gl>; 4],
    draw_vectors: bool,
    draw_control_points: bool,
}

impl<'gl> GregoryPatch<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        entities: &EntityCollection<'gl>,
        triangle: C0EdgeTriangle,
    ) -> Self {
        let mut gregory = Self {
            gl,
            name: ChangeableName::new("Gregory patch", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
            triangle,
            mesh: GregoryMesh::empty(gl),
            vector_meshes: Vec::new(),
            control_points_meshes: [
                PointCloud::new(gl, Vec::new()),
                PointCloud::new(gl, Vec::new()),
                PointCloud::new(gl, Vec::new()),
                PointCloud::new(gl, Vec::new()),
            ],
            draw_vectors: false,
            draw_control_points: false,
        };

        gregory.recalculate_mesh(entities);
        gregory
    }

    fn add_vector_mesh(&mut self, vec: &Vector3<f32>, point: &Point3<f32>, color: &Color) {
        self.vector_meshes.push(ColoredLineMesh::new(
            self.gl,
            vec![
                ColoredVertex::new(point.x, point.y, point.z, color.r, color.g, color.b),
                ColoredVertex::new(
                    point.x + vec.x,
                    point.y + vec.y,
                    point.z + vec.z,
                    color.r,
                    color.g,
                    color.b,
                ),
            ],
            vec![0, 1],
        ));

        self.vector_meshes
            .last_mut()
            .unwrap()
            .as_line_mesh_mut()
            .thickness(2.0);
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let patch0 = Self::patch_id_to_points(&self.triangle.0[0], entities);
        let patch1 = Self::patch_id_to_points(&self.triangle.0[1], entities);
        let patch2 = Self::patch_id_to_points(&self.triangle.0[2], entities);

        let triangle = GregoryTriangle::new([patch0, patch1, patch2]);
        self.vector_meshes.clear();

        for (u, p) in triangle
            .u_diff
            .iter()
            .flatten()
            .zip(triangle.twist_u_p.iter().flatten())
        {
            // Negated u derivatives are more useful
            self.add_vector_mesh(&-u, p, &Color::red());
        }

        for (v, p) in triangle
            .v_diff
            .iter()
            .flatten()
            .flatten()
            .zip(triangle.v_diff_p.iter().flatten().flatten())
        {
            self.add_vector_mesh(v, p, &Color::lblue());
        }

        for (t, p) in triangle
            .twist
            .iter()
            .flatten()
            .zip(triangle.twist_u_p.iter().flatten())
        {
            self.add_vector_mesh(t, p, &Color::lime());
        }

        self.control_points_meshes = [
            PointCloud::new(self.gl, Self::points(&triangle, 0)),
            PointCloud::new(self.gl, Self::points(&triangle, 1)),
            PointCloud::new(self.gl, Self::points(&triangle, 2)),
            PointCloud::new(self.gl, Self::common_points(&triangle)),
        ];

        self.mesh = GregoryMesh::new(self.gl, triangle.patches.into());
    }

    fn patch_id_to_points(patch: &C0Edge, entities: &EntityCollection<'gl>) -> BorderPatch {
        BorderPatch([
            [
                Self::point(patch.points[0][0], entities),
                Self::point(patch.points[0][1], entities),
                Self::point(patch.points[0][2], entities),
                Self::point(patch.points[0][3], entities),
            ],
            [
                Self::point(patch.points[1][0], entities),
                Self::point(patch.points[1][1], entities),
                Self::point(patch.points[1][2], entities),
                Self::point(patch.points[1][3], entities),
            ],
            [
                Self::point(patch.points[2][0], entities),
                Self::point(patch.points[2][1], entities),
                Self::point(patch.points[2][2], entities),
                Self::point(patch.points[2][3], entities),
            ],
            [
                Self::point(patch.points[3][0], entities),
                Self::point(patch.points[3][1], entities),
                Self::point(patch.points[3][2], entities),
                Self::point(patch.points[3][3], entities),
            ],
        ])
    }

    fn point(id: usize, entities: &EntityCollection<'gl>) -> Point3<f32> {
        entities[&id].borrow().location().unwrap()
    }

    fn points(triangle: &GregoryTriangle, patch: usize) -> Vec<Point3<f32>> {
        let patch = &triangle.patches[patch];

        patch.bottom[1..3]
            .iter()
            .chain(patch.u_inner.iter())
            .chain(patch.v_inner.iter())
            .chain([patch.top_sides[1]].iter())
            .chain([patch.bottom_sides[1]].iter())
            .copied()
            .collect()
    }

    fn common_points(triangle: &GregoryTriangle) -> Vec<Point3<f32>> {
        triangle
            .patches
            .iter()
            .map(|patch| patch.top.iter())
            .flatten()
            .copied()
            .collect()
    }

    fn draw_vectors(&self, camera: &Camera, premul: &Matrix4<f32>) {
        let program = self.shader_manager.program("cursor");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        for m in &self.vector_meshes {
            m.draw()
        }
    }

    fn draw_control_points(&self, camera: &Camera, premul: &Matrix4<f32>) {
        let program = self.shader_manager.program("point");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );

        unsafe { self.gl.enable(glow::PROGRAM_POINT_SIZE) };
        program.uniform_f32("point_size", 5.0);

        program.uniform_color("point_color", &Color::red());
        self.control_points_meshes[0].draw();

        program.uniform_color("point_color", &Color::green());
        self.control_points_meshes[1].draw();

        program.uniform_color("point_color", &Color::blue());
        self.control_points_meshes[2].draw();

        program.uniform_color("point_color", &Color::windows98());
        self.control_points_meshes[3].draw();
    }
}

impl<'gl> ReferentialEntity<'gl> for GregoryPatch<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("gregory_control");
        self.name_control_ui(ui);

        ui.checkbox("Draw vectors", &mut self.draw_vectors);
        ui.checkbox("Draw control points", &mut self.draw_control_points);

        uv_subdivision_ui(ui, &mut self.u_patch_divisions, &mut self.v_patch_divisions);

        ControlResult::default()
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &EntityCollection<'gl>,
    ) {
        self.recalculate_mesh(entities);
    }

    fn allow_deletion(&self, _deleted: &HashSet<usize>) -> bool {
        // Refuse deletion of any subscribed points or surfaces
        false
    }

    fn notify_about_reindexing(
        &mut self,
        changes: &HashMap<usize, usize>,
        entities: &EntityCollection<'gl>,
    ) {
        for edge in &mut self.triangle.0 {
            for old_id in edge.points.iter_mut().flatten() {
                if let Some(&new_id) = changes.get(old_id) {
                    *old_id = new_id;
                }
            }
        }

        self.recalculate_mesh(entities);
    }
}

impl<'gl> Drawable for GregoryPatch<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("gregory");
        let color = Color::for_draw_type(&draw_type);
        self.mesh.draw_with_program(
            program,
            camera,
            premul,
            &color,
            self.u_patch_divisions,
            self.v_patch_divisions,
        );

        if self.draw_vectors {
            self.draw_vectors(camera, premul);
        }

        if self.draw_control_points {
            self.draw_control_points(camera, premul);
        }
    }
}

impl<'gl> SceneObject for GregoryPatch<'gl> {}

impl<'gl> NamedEntity for GregoryPatch<'gl> {
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
            "objectType": "gregoryPatch",
            "name": self.name(),
        })
    }
}
