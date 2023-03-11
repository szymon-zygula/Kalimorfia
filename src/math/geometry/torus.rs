use super::parametric_form::ParametricForm;
use nalgebra::{Vector2, Point3};

pub struct Torus {
    pub inner_radius: f64,
    pub tube_radius: f64,
}

impl ParametricForm<2, 3> for Torus {
    const PARAMETER_BOUNDS: Vector2<(f64, f64)> = Vector2::new(
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
    );

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        Point3::new(
            (self.inner_radius + self.tube_radius * vec.x.cos()) * vec.y.cos(),
            (self.inner_radius + self.tube_radius * vec.x.cos()) * vec.y.sin(),
            self.tube_radius * vec.x.sin(),
        )
    }
}
