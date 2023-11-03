use crate::{
    camera::Camera,
    entities::{
        bezier_surface_args::*,
        bezier_utils::*,
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, Drawable, EntityCollection, NamedEntity, ReferentialEntity,
            SceneObject,
        },
        utils,
    },
    math::geometry::{
        bezier::{deboor_surface_to_bernstein, BezierSurface},
        gridable::Gridable,
        parametric_form::DifferentialParametricForm,
        surfaces::{ShiftedSurface, SurfaceC2},
    },
    primitives::color::Color,
    render::{
        bezier_surface_mesh::BezierSurfaceMesh, gl_drawable::GlDrawable, gl_texture::GlTexture,
        mesh::LinesMesh, shader_manager::ShaderManager, texture::Texture,
    },
    repositories::NameRepository,
};
use glow::HasContext;
use nalgebra::Matrix4;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
};

use super::{basic::IntersectionTexture, entity::Entity};

pub struct BezierSurfaceC2<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierSurfaceMesh<'gl>,
    shifted_mesh: LinesMesh<'gl>,
    deboor_polygon_mesh: LinesMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_deboor_polygon: bool,
    draw_bernstein_polygon: bool,
    draw_shifted: bool,

    points: Vec<Vec<usize>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,
    intersection_texture: IntersectionTexture<'gl>,

    pub u_patch_divisions: u32,
    pub v_patch_divisions: u32,

    pub surface: SurfaceC2,

    is_cylinder: bool,

    gk_mode: bool,
    wireframe: bool,

    displacement_texture: GlTexture<'gl>,
    color_texture: GlTexture<'gl>,
    normal_texture: GlTexture<'gl>,
}

impl<'gl> BezierSurfaceC2<'gl> {
    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        points: Vec<Vec<usize>>,
        entities: &EntityCollection<'gl>,
        args: BezierSurfaceArgs,
    ) -> Self {
        let is_cylinder = matches!(args, BezierSurfaceArgs::Cylinder(..));
        let [displacement_texture, color_texture, normal_texture] = Self::load_textures(gl);
        let mut s = Self {
            gl,
            mesh: BezierSurfaceMesh::empty(gl),
            shifted_mesh: LinesMesh::empty(gl),
            draw_shifted: false,
            deboor_polygon_mesh: LinesMesh::empty(gl),
            bernstein_polygon_mesh: LinesMesh::empty(gl),
            points,
            draw_deboor_polygon: false,
            draw_bernstein_polygon: false,
            name: ChangeableName::new("Bezier Surface C2", name_repo),
            shader_manager,
            u_patch_divisions: 3,
            v_patch_divisions: 3,
            intersection_texture: IntersectionTexture::empty(gl, is_cylinder, false),
            surface: SurfaceC2::null(),
            is_cylinder,
            gk_mode: false,
            wireframe: true,
            displacement_texture,
            color_texture,
            normal_texture,
        };

        s.recalculate_mesh(entities);

        s
    }

    fn load_textures(gl: &glow::Context) -> [GlTexture; 3] {
        [
            "textures/height.png",
            "textures/diffuse.png",
            "textures/normals.png",
        ]
        .map(|path| GlTexture::new(gl, &Texture::from_file(Path::new(path))))
    }

    pub fn wrapped_points(&self) -> Vec<Vec<usize>> {
        let mut points = self.points.clone();

        if self.is_cylinder {
            points.push(points[0].clone());
            points.push(points[1].clone());
            points.push(points[2].clone());
        }

        points
    }

    fn recalc_shifted_mesh(&mut self) {
        const SHIFT: f64 = 0.1;
        const RES: u32 = 30;
        let shifted = ShiftedSurface::new(&self.surface, SHIFT);

        let (vertices, indices) = shifted.grid(RES, RES);
        self.shifted_mesh =
            LinesMesh::new(self.gl, vertices.iter().map(|p| p.point).collect(), indices);
    }

    fn recalculate_mesh(&mut self, entities: &EntityCollection<'gl>) {
        let wrapped_points = self.wrapped_points();
        let deboor_points = point_ids_to_f64(&wrapped_points, entities);
        self.surface = SurfaceC2::from_points(deboor_points.clone(), self.is_cylinder, false);
        let bernstein_points = deboor_surface_to_bernstein(deboor_points);
        let bezier_surface = BezierSurface::new(bernstein_points);

        self.mesh = BezierSurfaceMesh::new(self.gl, bezier_surface.clone());

        if !self.wireframe {
            self.mesh.wireframe = false;
        }

        self.bernstein_polygon_mesh = grid_mesh(self.gl, bezier_surface.grid());

        let deboor_grid = create_grid(&self.points, entities, self.is_cylinder);
        self.deboor_polygon_mesh = grid_mesh(self.gl, &deboor_grid);
        self.recalc_shifted_mesh();
    }

    fn u_patches(&self) -> usize {
        if self.is_cylinder {
            self.points.len()
        } else {
            self.points.len() - 3
        }
    }

    fn v_patches(&self) -> usize {
        self.points.first().map_or(0, |first| first.len() - 3)
    }

    fn patch_control_points(&self, patch_u: usize, patch_v: usize) -> Vec<usize> {
        let mut points = Vec::new();

        for v in 0..4 {
            for u in 0..4 {
                points.push(self.points[(patch_u + u) % self.points.len()][patch_v + v]);
            }
        }

        points
    }

    fn json_patches(&self) -> Vec<serde_json::Value> {
        let mut patches = Vec::new();

        let u_patches = self.u_patches();
        let v_patches = self.v_patches();

        for patch_v in 0..v_patches {
            for patch_u in 0..u_patches {
                patches.push(serde_json::json!({
                    "objectType": "bezierPatchC2",
                    "name": "patch",
                    "controlPoints": utils::control_points_json(
                        &self.patch_control_points(patch_u, patch_v)
                    ),
                    "samples": {
                        "x": self.u_patch_divisions,
                        "y": self.v_patch_divisions
                    }
                }));
            }
        }

        patches
    }

    fn gk_control(&mut self, ui: &imgui::Ui) {
        ui.checkbox("Wireframe", &mut self.wireframe);
    }

    fn draw_gk(&self, premul: &Matrix4<f32>, camera: &Camera) {
        let program = self.shader_manager.program("gk_mode");
        program.enable();
        program.uniform_matrix_4_f32_slice("model", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice("projection", camera.projection_transform().as_slice());
        program.uniform_u32("subdivisions", self.u_patch_divisions);
        program.uniform_u32("u_patches", self.u_patches() as u32);
        program.uniform_u32("v_patches", self.v_patches() as u32);
        program.uniform_3_f32(
            "cam_pos",
            camera.position().x,
            camera.position().y,
            camera.position().z,
        );

        self.displacement_texture.bind_to_image_unit(0);
        self.color_texture.bind_to_image_unit(1);
        self.normal_texture.bind_to_image_unit(2);

        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
        }

        self.mesh.draw();
    }
}

impl<'gl> ReferentialEntity<'gl> for BezierSurfaceC2<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id(self.name());
        self.name_control_ui(ui);
        ui.checkbox("Draw de Boor polygon", &mut self.draw_deboor_polygon);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);
        ui.checkbox("Draw shifted surface", &mut self.draw_shifted);
        ui.checkbox("GK2 mode", &mut self.gk_mode);

        if self.gk_mode {
            self.gk_control(ui);
            subdivision_ui(ui, &mut self.u_patch_divisions, "Detail");
        } else {
            uv_subdivision_ui(ui, &mut self.u_patch_divisions, &mut self.v_patch_divisions);
        }

        self.intersection_texture.control_ui(ui);
        self.mesh.wireframe = self.wireframe;

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
        // Refuse deletion of any subscribed point
        false
    }

    fn notify_about_reindexing(
        &mut self,
        changes: &HashMap<usize, usize>,
        entities: &EntityCollection<'gl>,
    ) {
        for old_id in self.points.iter_mut().flatten() {
            if let Some(&new_id) = changes.get(old_id) {
                *old_id = new_id;
            }
        }

        self.recalculate_mesh(entities);
    }
}

impl<'gl> Drawable for BezierSurfaceC2<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        if self.gk_mode {
            self.draw_gk(premul, camera);
        } else {
            draw_bezier_surface(
                &self.mesh,
                self.u_patch_divisions,
                self.v_patch_divisions,
                &self.shader_manager,
                camera,
                premul,
                draw_type,
                self.intersection_texture.handle(),
                self.u_patches() as u32,
                self.v_patches() as u32,
            );
        }

        if self.draw_deboor_polygon {
            draw_polygon(
                &self.deboor_polygon_mesh,
                &self.shader_manager,
                camera,
                premul,
                draw_type,
            );
        }

        if self.draw_bernstein_polygon {
            draw_polygon(
                &self.bernstein_polygon_mesh,
                &self.shader_manager,
                camera,
                premul,
                draw_type,
            );
        }

        if self.draw_shifted {
            let program = self.shader_manager.program("spline");
            program.enable();
            program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
            program
                .uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
            program.uniform_matrix_4_f32_slice(
                "projection_transform",
                camera.projection_transform().as_slice(),
            );
            program.uniform_color("vertex_color", &Color::lblue());
            self.shifted_mesh.draw();
        }
    }
}

impl<'gl> SceneObject for BezierSurfaceC2<'gl> {
    fn set_intersection_texture(&mut self, texture: Texture) {
        self.intersection_texture =
            IntersectionTexture::new(self.gl, texture, self.is_cylinder, false);
    }

    fn intersection_texture(&self) -> Option<&IntersectionTexture<'gl>> {
        Some(&self.intersection_texture)
    }

    fn as_parametric_2_to_3(&self) -> Option<Box<dyn DifferentialParametricForm<2, 3>>> {
        Some(Box::new(self.surface.clone()))
    }
}

impl<'gl> NamedEntity for BezierSurfaceC2<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn set_similar_name(&mut self, name: &str) {
        self.name.set_similar_name(name)
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "objectType": "bezierSurfaceC2",
            "name": self.name(),
            "patches": self.json_patches(),
            "parameterWrapped": {
                "u": self.is_cylinder,
                "v": false,
            },
            "size": {
                "x": self.u_patches(),
                "y": self.v_patches(),
            }
        })
    }
}
