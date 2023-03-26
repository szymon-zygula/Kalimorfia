use nalgebra::Point3;

pub trait Curvable {
    fn curve(&self, samples: usize) -> Vec<Point3<f32>>;
}
