use super::parametric_form::DifferentialParametricForm;
use nalgebra::{Matrix3x2, Point3, Vector2};

#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub radius: f64,
}

impl Sphere {
    pub fn with_radius(radius: f64) -> Self {
        Sphere { radius }
    }
}

impl DifferentialParametricForm<2, 3> for Sphere {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (0.0, std::f64::consts::PI),
        )
    }

    fn wrapped(&self, _dim: usize) -> bool {
        true
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::new(
            self.radius * vec.x.cos() * vec.y.sin(),
            self.radius * vec.x.sin() * vec.y.sin(),
            self.radius * vec.y.cos(),
        )
    }

    fn jacobian(&self, _vec: &Vector2<f64>) -> Matrix3x2<f64> {
        unimplemented!("Sphere jacobians are not implemented")
    }
}
