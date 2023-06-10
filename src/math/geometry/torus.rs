use super::parametric_form::DifferentialParametricForm;
use nalgebra::{Matrix3x2, Matrix4, Point3, Vector2, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Torus {
    pub inner_radius: f64,
    pub tube_radius: f64,
}

impl Torus {
    pub fn with_radii(inner_radius: f64, tube_radius: f64) -> Torus {
        Torus {
            inner_radius,
            tube_radius,
        }
    }
}

impl DifferentialParametricForm<2, 3> for Torus {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (0.0, 2.0 * std::f64::consts::PI),
        )
    }

    fn wrapped(&self, _dim: usize) -> bool {
        true
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::new(
            (self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.cos(),
            self.tube_radius * vec.y.sin(),
            (self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.sin(),
        )
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        Matrix3x2::from_columns(&[
            Vector3::new(
                -(self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.sin(),
                0.0,
                (self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.cos(),
            ),
            Vector3::new(
                -self.tube_radius * vec.y.sin() * vec.x.cos(),
                self.tube_radius * vec.y.cos(),
                -self.tube_radius * vec.y.sin() * vec.x.sin(),
            ),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct AffineTorus {
    pub torus: Torus,
    pub transform: Matrix4<f64>,
}

impl AffineTorus {
    pub fn new(torus: Torus, transform: Matrix4<f64>) -> Self {
        Self { torus, transform }
    }
}

impl DifferentialParametricForm<2, 3> for AffineTorus {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        self.torus.bounds()
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.torus.wrapped(dim)
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::from_homogeneous(self.transform * self.torus.value(vec).to_homogeneous())
            .unwrap_or(Point3::origin())
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        self.transform.fixed_view::<3, 3>(0, 0) * self.torus.jacobian(vec)
    }
}
