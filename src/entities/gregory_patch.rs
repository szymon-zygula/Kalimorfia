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
    primitives::color::Color,
    render::{bezier_surface_mesh::GregoryMesh, shader_manager::ShaderManager},
    repositories::NameRepository,
};
use nalgebra::{Matrix4, Point3};
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
        };

        gregory.recalculate_mesh(entities);
        gregory
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let patch0 = Self::patch_id_to_points(&self.triangle.0[0], entities);
        let patch1 = Self::patch_id_to_points(&self.triangle.0[1], entities);
        let patch2 = Self::patch_id_to_points(&self.triangle.0[2], entities);

        let triangle = GregoryTriangle::new([patch0, patch1, patch2]);

        self.mesh = GregoryMesh::new(self.gl, triangle.0.into());
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
        )
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
