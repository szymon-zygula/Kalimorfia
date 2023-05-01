use nalgebra::{Matrix3, Matrix4, Point3, RealField, Vector3};
use num_traits::identities::Zero;

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

pub fn rotate_axis<T: RealField + Copy>(axis: Vector3<T>, angle: T) -> Matrix4<T> {
    if axis.is_zero() {
        return Matrix4::identity();
    }

    let cross_matrix = axis.normalize().cross_matrix();
    let rotation_matrix = Matrix3::identity()
        + cross_matrix * angle.sin()
        + cross_matrix * cross_matrix * (T::one() - angle.cos());

    rotation_matrix.to_homogeneous()
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

pub fn shear_xy_xz_yz<T: RealField + Copy>(xy: T, xz: T, yz: T) -> Matrix4<T> {
    let mut shear = Matrix4::identity();
    shear[(0, 1)] = xy;
    shear[(0, 2)] = xz;
    shear[(1, 2)] = yz;

    shear
}

pub fn inverse_shear_xy_xz_yz<T: RealField + Copy>(xy: T, xz: T, yz: T) -> Matrix4<T> {
    let mut inverse_shear = Matrix4::identity();
    inverse_shear[(0, 1)] = -xy;
    inverse_shear[(0, 2)] = xy * yz - xz;
    inverse_shear[(1, 2)] = -yz;

    inverse_shear
}

pub fn uniform_scale<T: RealField + Copy>(sxyz: T) -> Matrix4<T> {
    scale(sxyz, sxyz, sxyz)
}

pub fn projection<T: RealField + Copy>(
    fov: T,
    aspect_ratio: T,
    near_plane: T,
    far_plane: T,
) -> Matrix4<T> {
    let mut projection_matrix = Matrix4::zeros();

    let ctg_fov_over_2 = T::one() / (fov * T::from_f32(0.5).unwrap()).tan();
    let view_distance = far_plane - near_plane;

    projection_matrix[(0, 0)] = ctg_fov_over_2 / aspect_ratio;
    projection_matrix[(1, 1)] = ctg_fov_over_2;
    projection_matrix[(2, 2)] = -(far_plane + near_plane) / view_distance;
    projection_matrix[(2, 3)] = -T::from_f32(2.0).unwrap() * far_plane * near_plane / view_distance;
    projection_matrix[(3, 2)] = -T::one();

    projection_matrix
}

pub fn six_planes_projection<T: RealField + Copy>(
    near_plane: T,
    far_plane: T,
    top_plane: T,
    bottom_plane: T,
    left_plane: T,
    right_plane: T,
) -> Matrix4<T> {
    let mut projection_matrix = Matrix4::zeros();
    let two = T::from_f64(2.0).unwrap();

    projection_matrix[(0, 0)] = two * near_plane / (right_plane - left_plane);
    projection_matrix[(0, 2)] = (right_plane + left_plane) / (right_plane - left_plane);
    projection_matrix[(1, 1)] = (two * near_plane) / (top_plane - bottom_plane);
    projection_matrix[(1, 2)] = (top_plane + bottom_plane) / (top_plane - bottom_plane);
    projection_matrix[(2, 2)] = (far_plane + near_plane) / (far_plane - near_plane);
    projection_matrix[(2, 3)] = (-two * far_plane * near_plane) / (far_plane - near_plane);
    projection_matrix[(3, 2)] = T::one();

    projection_matrix
}

pub fn stereo_projection<T: RealField + Copy>() -> (Matrix4<T>, Matrix4<T>) {
    todo!()
}

pub fn inverse_projection<T: RealField + Copy>(
    fov: T,
    aspect_ratio: T,
    near_plane: T,
    far_plane: T,
) -> Matrix4<T> {
    let mut projection_matrix = Matrix4::zeros();

    let tan_fov_over_2 = (fov * T::from_f32(0.5).unwrap()).tan();
    let view_distance = far_plane - near_plane;

    projection_matrix[(0, 0)] = tan_fov_over_2 * aspect_ratio;
    projection_matrix[(1, 1)] = tan_fov_over_2;
    projection_matrix[(2, 3)] = -T::one();
    projection_matrix[(3, 2)] =
        view_distance / (-T::from_f32(2.0).unwrap() * far_plane * near_plane);
    projection_matrix[(3, 3)] =
        -(far_plane + near_plane) / view_distance * projection_matrix[(3, 2)];

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
