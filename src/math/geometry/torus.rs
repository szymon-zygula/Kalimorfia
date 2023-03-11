use super::parametric_form::ParametricForm;
use nalgebra::{Vector2, Vector3};

pub struct Torus {
    pub inner_radius: f64,
    pub tube_radius: f64,
}

impl ParametricForm<2, 3> for Torus {
    fn parametric(&self, vec: &Vector2<f64>) -> Vector3<f64> {
        Vector3::new(
            (self.inner_radius + self.tube_radius * vec.x.cos()) * vec.y.cos(),
            (self.inner_radius + self.tube_radius * vec.x.cos()) * vec.y.sin(),
            self.tube_radius * vec.x.sin(),
        )
    }
}
