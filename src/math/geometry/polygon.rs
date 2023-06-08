use super::parametric_form::ParametricForm;
use nalgebra::{Point3, Vector1};

#[derive(Clone, Debug)]
pub struct Polygon {
    points: Vec<Point3<f64>>,
}

impl Polygon {
    pub fn through_points(points: Vec<Point3<f64>>) -> Self {
        Self { points }
    }
}

impl ParametricForm<1, 3> for Polygon {
    fn bounds(&self) -> Vector1<(f64, f64)> {
        Vector1::new((0.0, 1.0))
    }

    fn parametric(&self, vec: &Vector1<f64>) -> Point3<f64> {
        let line_idx = if vec.x == 1.0 {
            self.points.len() - 2
        } else {
            (vec.x * (self.points.len() - 1) as f64).floor() as usize
        };

        let line_t = (self.points.len() - 1) as f64 * vec.x - line_idx as f64;
        (self.points[line_idx].coords * line_t + self.points[line_idx + 1].coords * (1.0 - line_t))
            .into()
    }
}
