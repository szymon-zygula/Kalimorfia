use crate::{
    camera::Camera,
    entities::{
        changeable_name::ChangeableName,
        entity::{
            ControlResult, DrawType, Drawable, EntityCollection, NamedEntity, ReferentialEntity,
            SceneObject,
        },
    },
    math::{geometry::bezier::BezierSurface, utils::point_32_to_64},
    primitives::color::Color,
    render::{
        bezier_surface_mesh::BezierSurfaceMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
    repositories::NameRepository,
};
use itertools::Itertools;
use nalgebra::Matrix4;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Copy, Clone, Debug)]
pub struct BezierSurfaceArgs {
    pub x_length: f32,
    pub z_length: f32,

    pub x_patches: i32,
    pub z_patches: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct BezierCylinderArgs {
    pub length: f32,
    pub radius: f32,

    pub around_patches: i32,
    pub along_patches: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum BezierSurfaceC0Args {
    Surface(BezierSurfaceArgs),
    Cylinder(BezierCylinderArgs),
}

impl BezierSurfaceC0Args {
    const MIN_PATCHES: i32 = 1;
    const MAX_PATCHES: i32 = 30;
    const MIN_LENGTH: f32 = 0.1;
    const MAX_LENGTH: f32 = 10.0;

    pub fn new_surface() -> Self {
        Self::Surface(BezierSurfaceArgs {
            x_length: 1.0,
            z_length: 1.0,

            x_patches: 1,
            z_patches: 1,
        })
    }

    pub fn new_cylinder() -> Self {
        Self::Cylinder(BezierCylinderArgs {
            length: 1.0,
            radius: 1.0,
            around_patches: 1,
            along_patches: 1,
        })
    }

    pub fn clamp_values(&mut self) {
        match self {
            BezierSurfaceC0Args::Surface(surface) => {
                Self::clamp_patches(&mut surface.x_patches);
                Self::clamp_patches(&mut surface.z_patches);
                Self::clamp_length(&mut surface.x_length);
                Self::clamp_length(&mut surface.z_length);
            }
            BezierSurfaceC0Args::Cylinder(cyllinder) => {
                Self::clamp_patches(&mut cyllinder.around_patches);
                Self::clamp_patches(&mut cyllinder.along_patches);
                Self::clamp_length(&mut cyllinder.length);
                Self::clamp_length(&mut cyllinder.radius);
            }
        }
    }

    fn clamp_patches(patches: &mut i32) {
        if *patches < Self::MIN_PATCHES {
            *patches = Self::MIN_PATCHES;
        } else if *patches > Self::MAX_PATCHES {
            *patches = Self::MAX_PATCHES;
        }
    }

    fn clamp_length(length: &mut f32) {
        if *length < Self::MIN_LENGTH {
            *length = Self::MIN_LENGTH;
        } else if *length > Self::MAX_LENGTH {
            *length = Self::MAX_LENGTH;
        }
    }
}

pub struct BezierSurfaceC0<'gl> {
    gl: &'gl glow::Context,

    mesh: BezierSurfaceMesh<'gl>,
    bernstein_polygon_mesh: LinesMesh<'gl>,

    draw_bernstein_polygon: bool,

    points: Vec<Vec<usize>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    name: ChangeableName,

    u_patch_divisions: u32,
    v_patch_divisions: u32,

    bezier_surface: BezierSurface,
    is_cyllinder: bool,
}

impl<'gl> BezierSurfaceC0<'gl> {
    const MAX_SUBDIVISIONS: u32 = 25;
    const MIN_SUBDIVISIONS: u32 = 1;

    pub fn new(
        gl: &'gl glow::Context,
        name_repo: Rc<RefCell<dyn NameRepository>>,
        shader_manager: Rc<ShaderManager<'gl>>,
        points: Vec<Vec<usize>>,
        entities: &EntityCollection<'gl>,
        args: BezierSurfaceC0Args,
    ) -> Self {
        let is_cylinder = matches!(args, BezierSurfaceC0Args::Cylinder(..));
        let bezier_surface = Self::create_bezier_surface(&points, entities, is_cylinder);

        Self {
            gl,
            mesh: BezierSurfaceMesh::new(gl, bezier_surface.clone()),
            points,
            bernstein_polygon_mesh: Self::bernstein_mesh(gl, &bezier_surface),
            draw_bernstein_polygon: false,
            name: ChangeableName::new("Bezier Surface C0", name_repo),
            shader_manager,
            u_patch_divisions: 2,
            v_patch_divisions: 2,
            bezier_surface,
            is_cyllinder: is_cylinder,
        }
    }

    fn create_bezier_surface(
        points: &[Vec<usize>],
        entities: &EntityCollection<'gl>,
        is_cyllinder: bool,
    ) -> BezierSurface {
        let mut points: Vec<Vec<_>> = points
            .iter()
            .map(|u_row| {
                u_row
                    .iter()
                    .map(|p| point_32_to_64(entities[p].borrow().location().unwrap()))
                    .collect()
            })
            .collect();

        if is_cyllinder {
            for u_row in &mut points {
                u_row.push(u_row[0]);
            }
        }

        BezierSurface::new(points)
    }

    fn recalculate_surface(&mut self, entities: &EntityCollection<'gl>) {
        self.bezier_surface =
            Self::create_bezier_surface(&self.points, entities, self.is_cyllinder);
    }

    fn bernstein_mesh(gl: &'gl glow::Context, bezier_surface: &BezierSurface) -> LinesMesh<'gl> {
        let vertices = bezier_surface.flat_points();
        let indices = (0..bezier_surface.u_points())
            .tuple_windows()
            .cartesian_product(0..bezier_surface.v_points())
            .flat_map(|((u1, u2), v)| {
                [
                    bezier_surface.flat_idx(u1, v) as u32,
                    bezier_surface.flat_idx(u2, v) as u32,
                ]
            })
            .chain(
                (0..bezier_surface.u_points())
                    .cartesian_product((0..bezier_surface.v_points()).tuple_windows())
                    .flat_map(|(u, (v1, v2))| {
                        [
                            bezier_surface.flat_idx(u, v1) as u32,
                            bezier_surface.flat_idx(u, v2) as u32,
                        ]
                    }),
            )
            .collect();

        LinesMesh::new(gl, vertices, indices)
    }

    fn recalculate_mesh(&mut self) {
        self.mesh = BezierSurfaceMesh::new(self.gl, self.bezier_surface.clone());
        self.bernstein_polygon_mesh = Self::bernstein_mesh(self.gl, &self.bezier_surface);
    }

    fn draw_surface(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        let program = self.shader_manager.program("surface");
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

impl<'gl> ReferentialEntity<'gl> for BezierSurfaceC0<'gl> {
    fn control_referential_ui(
        &mut self,
        ui: &imgui::Ui,
        _controller_id: usize,
        _entities: &EntityCollection<'gl>,
        _subscriptions: &mut HashMap<usize, HashSet<usize>>,
    ) -> ControlResult {
        let _token = ui.push_id("c0_surface_control");
        self.name_control_ui(ui);
        ui.checkbox("Draw Bernstein polygon", &mut self.draw_bernstein_polygon);

        ui.slider_config(
            "U patch subdivisions",
            Self::MIN_SUBDIVISIONS,
            Self::MAX_SUBDIVISIONS,
        )
        .flags(imgui::SliderFlags::NO_INPUT)
        .build(&mut self.u_patch_divisions);

        ui.slider_config(
            "V patch subdivisions",
            Self::MIN_SUBDIVISIONS,
            Self::MAX_SUBDIVISIONS,
        )
        .flags(imgui::SliderFlags::NO_INPUT)
        .build(&mut self.v_patch_divisions);

        ControlResult::default()
    }

    fn notify_about_modification(
        &mut self,
        _modified: &HashSet<usize>,
        entities: &EntityCollection<'gl>,
    ) {
        self.recalculate_surface(entities);
        self.recalculate_mesh();
    }

    fn allow_deletion(&self, _deleted: &HashSet<usize>) -> bool {
        // Refuse deletion of any subscribed point
        false
    }
}

impl<'gl> Drawable for BezierSurfaceC0<'gl> {
    fn draw(&self, camera: &Camera, premul: &Matrix4<f32>, draw_type: DrawType) {
        self.draw_surface(camera, premul, draw_type);

        let program = self.shader_manager.program("spline");
        program.enable();
        program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice(
            "projection_transform",
            camera.projection_transform().as_slice(),
        );
        program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));

        if self.draw_bernstein_polygon {
            self.bernstein_polygon_mesh.draw();
        }
    }
}

impl<'gl> SceneObject for BezierSurfaceC0<'gl> {}

impl<'gl> NamedEntity for BezierSurfaceC0<'gl> {
    fn name(&self) -> String {
        self.name.name()
    }

    fn name_control_ui(&mut self, ui: &imgui::Ui) {
        self.name.name_control_ui(ui);
    }
}
