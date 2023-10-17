use super::parametric_form::DifferentialParametricForm;
use nalgebra::{Matrix3x2, Point3, Vector2};

#[derive(Copy, Clone, Debug)]
pub struct Cylinder {
    pub radius: f64,
    pub length: f64,
}

impl Cylinder {
    pub fn new(radius: f64, length: f64) -> Self {
        Self { radius, length }
    }
}

impl DifferentialParametricForm<2, 3> for Cylinder {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (-0.1, 1.1), // [0.0, 1.0] for the walls, the rest for the tops
        )
    }

    fn wrapped(&self, dim: usize) -> bool {
        dim == 0
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        let r = self.radius
            * if vec.y < 0.0 {
                10.0 * (vec.y + 0.1)
            } else if vec.y > 1.0 {
                10.0 * (1.1 - vec.y)
            } else {
                1.0
            };

        Point3::new(
            r * vec.x.cos(),
            r * vec.x.sin(),
            self.length * vec.y.clamp(0.0, 1.0),
        )
    }

    fn jacobian(&self, _vec: &Vector2<f64>) -> Matrix3x2<f64> {
        unimplemented!("Cylinder jacobians are not implemented")
    }
}
