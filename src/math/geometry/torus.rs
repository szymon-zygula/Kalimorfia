use super::parametric_form::ParametricForm;
use nalgebra::{Matrix4, Point3, Vector2};

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

impl ParametricForm<2, 3> for Torus {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new(
            (0.0, 2.0 * std::f64::consts::PI),
            (0.0, 2.0 * std::f64::consts::PI),
        )
    }

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::new(
            (self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.cos(),
            self.tube_radius * vec.y.sin(),
            (self.inner_radius + self.tube_radius * vec.y.cos()) * vec.x.sin(),
        )
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

impl ParametricForm<2, 3> for AffineTorus {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        self.torus.bounds()
    }

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::from_homogeneous(self.transform * self.torus.parametric(vec).to_homogeneous())
            .unwrap_or(Point3::origin())
    }
}
