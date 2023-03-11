use nalgebra::{SMatrix, Vector3};

pub type AffineTransform = SMatrix<f64, 4, 4>;

pub fn rotate_x(angle: f64) -> AffineTransform {
    let mut rot_x = AffineTransform::zeros();

    rot_x[(0, 0)] = 1.0;
    rot_x[(3, 3)] = 1.0;

    rot_x[(1, 1)] = angle.cos();
    rot_x[(1, 2)] = -angle.sin();
    rot_x[(2, 1)] = angle.sin();
    rot_x[(2, 2)] = angle.cos();

    rot_x
}

pub fn rotate_y(angle: f64) -> AffineTransform {
    let mut rot_y = AffineTransform::zeros();

    rot_y[(1, 1)] = 1.0;
    rot_y[(3, 3)] = 1.0;

    rot_y[(0, 0)] = angle.cos();
    rot_y[(0, 2)] = angle.sin();
    rot_y[(2, 0)] = -angle.sin();
    rot_y[(2, 2)] = angle.cos();

    rot_y
}

pub fn rotate_z(angle: f64) -> AffineTransform {
    let mut rot_z = AffineTransform::zeros();

    rot_z[(2, 2)] = 1.0;
    rot_z[(3, 3)] = 1.0;

    rot_z[(0, 0)] = angle.cos();
    rot_z[(0, 1)] = -angle.sin();
    rot_z[(1, 0)] = angle.sin();
    rot_z[(1, 1)] = angle.cos();

    rot_z
}

pub fn translate(vector: Vector3<f64>) -> AffineTransform {
    let mut translation = AffineTransform::identity();

    translation[(0, 3)] = vector[0];
    translation[(1, 3)] = vector[1];
    translation[(2, 3)] = vector[2];

    translation
}

pub fn scale(sx: f64, sy: f64, sz: f64) -> AffineTransform {
    let mut scaling = AffineTransform::zeros();

    scaling[(0, 0)] = sx;
    scaling[(1, 1)] = sy;
    scaling[(2, 2)] = sz;
    scaling[(3, 3)] = 1.0;

    scaling
}
