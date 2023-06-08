use crate::{
    camera::Camera,
    primitives::color::Color,
    render::{gl_drawable::GlDrawable, point_cloud::PointCloud, shader_manager::ShaderManager},
};
use glow::HasContext;
use nalgebra::{Matrix4, Point3};

pub fn slice_as_raw<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        )
    }
}

/// For use with e.g. `Vec::retain`.
pub fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = -1;
    move |item| {
        i += 1;
        f(i as usize, item)
    }
}

pub fn transpose_vector<T: Clone>(vec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut transpose = vec![Vec::<Option<T>>::new(); vec[0].len()];

    for i in 0..vec[0].len() {
        transpose[i].resize(vec.len(), None);
        for j in 0..vec.len() {
            transpose[i][j] = Some(vec[j][i].clone());
        }
    }

    transpose
        .into_iter()
        .map(|v| v.into_iter().map(|e| e.unwrap()).collect())
        .collect()
}

pub fn debug_point(
    gl: &glow::Context,
    camera: &Camera,
    point: Point3<f32>,
    shader_manager: &ShaderManager,
) {
    let program = shader_manager.program("point");
    program.enable();
    program.uniform_matrix_4_f32_slice("model_transform", Matrix4::identity().as_slice());
    program.uniform_matrix_4_f32_slice("view_transform", camera.view_transform().as_slice());
    program.uniform_matrix_4_f32_slice(
        "projection_transform",
        camera.projection_transform().as_slice(),
    );

    unsafe { gl.enable(glow::PROGRAM_POINT_SIZE) };
    program.uniform_f32("point_size", 7.0);

    program.uniform_color("point_color", &Color::red());

    PointCloud::new(gl, vec![point]).draw();
}
