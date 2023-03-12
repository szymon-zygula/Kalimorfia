use nalgebra::{Matrix4, Point3, RealField, Vector3};

pub fn rotate_x<T: RealField + Copy>(angle: T) -> Matrix4<T> {
    let mut rot_x = Matrix4::zeros();

    rot_x[(0, 0)] = T::one();
    rot_x[(3, 3)] = T::one();

    rot_x[(1, 1)] = angle.cos();
    rot_x[(1, 2)] = -angle.sin();
    rot_x[(2, 1)] = angle.sin();
    rot_x[(2, 2)] = angle.cos();

    rot_x
}

pub fn rotate_y<T: RealField + Copy>(angle: T) -> Matrix4<T> {
    let mut rot_y = Matrix4::zeros();

    rot_y[(1, 1)] = T::one();
    rot_y[(3, 3)] = T::one();

    rot_y[(0, 0)] = angle.cos();
    rot_y[(0, 2)] = angle.sin();
    rot_y[(2, 0)] = -angle.sin();
    rot_y[(2, 2)] = angle.cos();

    rot_y
}

pub fn rotate_z<T: RealField + Copy>(angle: T) -> Matrix4<T> {
    let mut rot_z = Matrix4::zeros();

    rot_z[(2, 2)] = T::one();
    rot_z[(3, 3)] = T::one();

    rot_z[(0, 0)] = angle.cos();
    rot_z[(0, 1)] = -angle.sin();
    rot_z[(1, 0)] = angle.sin();
    rot_z[(1, 1)] = angle.cos();

    rot_z
}

pub fn translate<T: RealField + Copy>(vector: Vector3<T>) -> Matrix4<T> {
    let mut translation = Matrix4::identity();

    translation[(0, 3)] = vector[0];
    translation[(1, 3)] = vector[1];
    translation[(2, 3)] = vector[2];

    translation
}

pub fn scale<T: RealField + Copy>(sx: T, sy: T, sz: T) -> Matrix4<T> {
    let mut scaling = Matrix4::zeros();

    scaling[(0, 0)] = sx;
    scaling[(1, 1)] = sy;
    scaling[(2, 2)] = sz;
    scaling[(3, 3)] = T::one();

    scaling
}

pub fn projection<T: RealField + Copy>(
    fov: T,
    aspect_ration: T,
    near_plane: T,
    far_plane: T,
) -> Matrix4<T> {
    let mut projection_matrix = Matrix4::zeros();

    let ctg_fov_over_2 = T::one() / (fov * T::from_f32(0.5).unwrap()).tan();
    let view_distance = far_plane - near_plane;

    projection_matrix[(0, 0)] = ctg_fov_over_2 / aspect_ration;
    projection_matrix[(1, 1)] = ctg_fov_over_2;
    projection_matrix[(2, 2)] = -(far_plane + near_plane) / view_distance;
    projection_matrix[(2, 3)] = -T::from_f32(2.0).unwrap() * far_plane * near_plane / view_distance;
    projection_matrix[(3, 2)] = -T::one();

    projection_matrix
}

pub fn look_at<T: RealField + Copy>(
    observation: Point3<T>,
    camera: Point3<T>,
    up: Vector3<T>,
) -> Matrix4<T> {
    let to_camera = (camera - observation).normalize();
    let right = up.cross(&to_camera).normalize();
    let head = to_camera.cross(&right);

    Matrix4::from_columns(&[
        right.to_homogeneous(),
        head.to_homogeneous(),
        to_camera.to_homogeneous(),
        camera.to_homogeneous(),
    ])
    .try_inverse()
    .unwrap()
}
