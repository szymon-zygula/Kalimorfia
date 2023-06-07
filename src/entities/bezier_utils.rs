use crate::{
    camera::Camera,
    entities::entity::{DrawType, EntityCollection},
    math::{
        geometry::bezier::{BezierSurface, PointsGrid},
        utils::point_32_to_64,
    },
    primitives::color::Color,
    render::{
        bezier_surface_mesh::BezierSurfaceMesh, gl_drawable::GlDrawable, mesh::LinesMesh,
        shader_manager::ShaderManager,
    },
};
use itertools::Itertools;
use nalgebra::{Matrix4, Point3};

pub const MAX_SUBDIVISIONS: u32 = 50;
pub const MIN_SUBDIVISIONS: u32 = 1;

pub fn point_ids_to_f64(
    points: &[Vec<usize>],
    entities: &EntityCollection,
) -> Vec<Vec<Point3<f64>>> {
    points
        .iter()
        .map(|u_row| {
            u_row
                .iter()
                .map(|p| point_32_to_64(entities[p].borrow().location().unwrap()))
                .collect()
        })
        .collect()
}

pub fn create_bezier_surface(
    points: &[Vec<usize>],
    entities: &EntityCollection,
    is_cylinder: bool,
) -> BezierSurface {
    let mut points: Vec<Vec<_>> = point_ids_to_f64(points, entities);

    if is_cylinder {
        points.push(points[0].clone());
    }

    BezierSurface::new(points)
}

pub fn create_grid(
    points: &[Vec<usize>],
    entities: &EntityCollection,
    is_cylinder: bool,
) -> PointsGrid {
    let mut points: Vec<Vec<_>> = point_ids_to_f64(points, entities);

    if is_cylinder {
        points.push(points[0].clone());
    }

    PointsGrid::new(points)
}

pub fn draw_bezier_surface(
    mesh: &BezierSurfaceMesh,
    u_patch_divisions: u32,
    v_patch_divisions: u32,
    shader_manager: &ShaderManager,
    camera: &Camera,
    premul: &Matrix4<f32>,
    draw_type: DrawType,
) {
    let program = shader_manager.program("surface");
    let color = Color::for_draw_type(&draw_type);
    mesh.draw_with_program(
        program,
        camera,
        premul,
        &color,
        u_patch_divisions,
        v_patch_divisions,
    )
}

pub fn draw_polygon(
    polygon_mesh: &LinesMesh,
    shader_manager: &ShaderManager,
    camera: &Camera,
    premul: &Matrix4<f32>,
    draw_type: DrawType,
) {
    let program = shader_manager.program("spline");
    program.enable();
    program.uniform_matrix_4_f32_slice("model_transform", premul.as_slice());
    program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
    program.uniform_matrix_4_f32_slice(
        "projection_transform",
        camera.projection_transform().as_slice(),
    );
    program.uniform_color("vertex_color", &Color::for_draw_type(&draw_type));

    polygon_mesh.draw();
}

pub fn grid_mesh<'gl>(gl: &'gl glow::Context, grid: &PointsGrid) -> LinesMesh<'gl> {
    let vertices = grid.flat_points();
    let indices = (0..grid.u_points())
        .tuple_windows()
        .cartesian_product(0..grid.v_points())
        .flat_map(|((u1, u2), v)| [grid.flat_idx(u1, v) as u32, grid.flat_idx(u2, v) as u32])
        .chain(
            (0..grid.u_points())
                .cartesian_product((0..grid.v_points()).tuple_windows())
                .flat_map(|(u, (v1, v2))| {
                    [grid.flat_idx(u, v1) as u32, grid.flat_idx(u, v2) as u32]
                }),
        )
        .collect();

    LinesMesh::new(gl, vertices, indices)
}

pub fn uv_subdivision_ui(ui: &imgui::Ui, u_patch_divisions: &mut u32, v_patch_divisions: &mut u32) {
    ui.slider_config("U patch subdivisions", MIN_SUBDIVISIONS, MAX_SUBDIVISIONS)
        .flags(imgui::SliderFlags::NO_INPUT)
        .build(u_patch_divisions);

    ui.slider_config("V patch subdivisions", MIN_SUBDIVISIONS, MAX_SUBDIVISIONS)
        .flags(imgui::SliderFlags::NO_INPUT)
        .build(v_patch_divisions);
}
