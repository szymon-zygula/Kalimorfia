use nalgebra::{Matrix4, Vector3};

pub fn rotate_x(angle: f64) -> Matrix4<f64> {
    let mut rot_x = Matrix4::zeros();

    rot_x[(0, 0)] = 1.0;
    rot_x[(3, 3)] = 1.0;

    rot_x[(1, 1)] = angle.cos();
    rot_x[(1, 2)] = -angle.sin();
    rot_x[(2, 1)] = angle.sin();
    rot_x[(2, 2)] = angle.cos();

    rot_x
}

pub fn rotate_y(angle: f64) -> Matrix4<f64> {
    let mut rot_y = Matrix4::zeros();

    rot_y[(1, 1)] = 1.0;
    rot_y[(3, 3)] = 1.0;

    rot_y[(0, 0)] = angle.cos();
    rot_y[(0, 2)] = angle.sin();
    rot_y[(2, 0)] = -angle.sin();
    rot_y[(2, 2)] = angle.cos();

    rot_y
}

pub fn rotate_z(angle: f64) -> Matrix4<f64> {
    let mut rot_z = Matrix4::zeros();

    rot_z[(2, 2)] = 1.0;
    rot_z[(3, 3)] = 1.0;

    rot_z[(0, 0)] = angle.cos();
    rot_z[(0, 1)] = -angle.sin();
    rot_z[(1, 0)] = angle.sin();
    rot_z[(1, 1)] = angle.cos();

    rot_z
}

pub fn translate(vector: Vector3<f64>) -> Matrix4<f64> {
    let mut translation = Matrix4::identity();

    translation[(0, 3)] = vector[0];
    translation[(1, 3)] = vector[1];
    translation[(2, 3)] = vector[2];

    translation
}

pub fn scale(sx: f64, sy: f64, sz: f64) -> Matrix4<f64> {
    let mut scaling = Matrix4::zeros();

    scaling[(0, 0)] = sx;
    scaling[(1, 1)] = sy;
    scaling[(2, 2)] = sz;
    scaling[(3, 3)] = 1.0;

    scaling
}

pub fn projection(fov: f32, aspect_ration: f32, near_plane: f32, far_plane: f32) -> Matrix4<f32> {
    let mut projection_matrix = Matrix4::zeros();

    let ctg_fov_over_2 = 1.0 / (fov / 2.0).tan();
    let view_distance = far_plane - near_plane;

    projection_matrix[(0, 0)] = ctg_fov_over_2 / aspect_ration;
    projection_matrix[(1, 1)] = ctg_fov_over_2;
    projection_matrix[(2, 2)] = (far_plane + near_plane) / view_distance;
    projection_matrix[(2, 3)] = -2.0 * far_plane * near_plane / view_distance;
    projection_matrix[(3, 2)] = 1.0;

    projection_matrix
}
