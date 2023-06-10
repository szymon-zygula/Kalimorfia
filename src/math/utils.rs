use nalgebra::{Const, Matrix4, OPoint, Point3, RealField, Scalar, Vector3};

pub fn point_64_to_32(p: Point3<f64>) -> Point3<f32> {
    Point3::new(p.x as f32, p.y as f32, p.z as f32)
}

pub fn point_32_to_64(p: Point3<f32>) -> Point3<f64> {
    Point3::new(p.x as f64, p.y as f64, p.z as f64)
}

pub fn vec_64_to_32(p: Vector3<f64>) -> Vector3<f32> {
    Vector3::new(p.x as f32, p.y as f32, p.z as f32)
}

pub fn vec_32_to_64(p: Vector3<f32>) -> Vector3<f64> {
    Vector3::new(p.x as f64, p.y as f64, p.z as f64)
}

pub fn mat_32_to_64(mat: Matrix4<f32>) -> Matrix4<f64> {
    Matrix4::from_fn(|i, j| mat[(i, j)] as f64)
}

pub fn point_avg<const DIM: usize, T: std::fmt::Debug + Scalar + RealField>(
    point_0: OPoint<T, Const<DIM>>,
    point_1: OPoint<T, Const<DIM>>,
) -> OPoint<T, Const<DIM>> {
    OPoint::from((point_0.coords + point_1.coords) * T::from_f64(0.5).unwrap())
}
