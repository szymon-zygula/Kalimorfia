use nalgebra::Point3;

pub struct InterGuide {
    pub id_0: usize,
    pub id_1: usize,
    pub guide: Point3<f64>,
    pub shifted_sign_0: f64,
    pub shifted_sign_1: f64
}

pub struct InterPlaneGuide {
    pub id: usize,
    pub guide: Point3<f64>,
}
