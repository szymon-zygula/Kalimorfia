use super::parametric_form::ParametricForm;
use crate::math::bernstein_polynomial::BernsteinPolynomial;
use nalgebra::{Point3, Vector1};

#[derive(Clone, Debug)]
pub struct BezierCurve {
    x_t: BernsteinPolynomial<f64>,
    y_t: BernsteinPolynomial<f64>,
    z_t: BernsteinPolynomial<f64>,
}

impl BezierCurve {
    pub fn through_points(points: Vec<Point3<f64>>) -> Self {
        Self {
            x_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.x).collect()),
            y_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.y).collect()),
            z_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.z).collect()),
        }
    }
}

impl ParametricForm<1, 3> for BezierCurve {
    const PARAMETER_BOUNDS: Vector1<(f64, f64)> = Vector1::new((0.0, 1.0));

    fn parametric(&self, vec: &Vector1<f64>) -> Point3<f64> {
        Point3::new(
            self.x_t.value(vec.x),
            self.y_t.value(vec.x),
            self.z_t.value(vec.x),
        )
    }
}

#[derive(Clone, Debug)]
pub struct CubicSplineC0 {
    curves: Vec<BezierCurve>,
}

impl CubicSplineC0 {
    pub fn through_points(points: Vec<Point3<f64>>) -> Self {
        assert_ne!(points.len(), 0);
        let curve_count = (points.len() - 1) / 3 + 1;
        let mut curves = Vec::with_capacity(curve_count);

        for i in 0..(curve_count - 1) {
            curves.push(BezierCurve::through_points(vec![
                points[i * 3],
                points[i * 3 + 1],
                points[i * 3 + 2],
                points[i * 3 + 3],
            ]));
        }

        let i = curve_count - 1;
        curves.push(match (points.len() - 1) % 3 {
            0 => BezierCurve::through_points(vec![points[i * 3]]),
            1 => BezierCurve::through_points(vec![points[i * 3], points[i * 3 + 1]]),
            2 => BezierCurve::through_points(vec![
                points[i * 3],
                points[i * 3 + 1],
                points[i * 3 + 2],
            ]),
            _ => panic!("Invalid remainder"),
        });

        Self { curves }
    }
}

impl ParametricForm<1, 3> for CubicSplineC0 {
    const PARAMETER_BOUNDS: Vector1<(f64, f64)> = Vector1::new((0.0, 1.0));

    fn parametric(&self, vec: &Vector1<f64>) -> Point3<f64> {
        let curve_idx = if vec.x == 1.0 {
            self.curves.len() - 1
        } else {
            (vec.x * self.curves.len() as f64).floor() as usize
        };

        let curve_t = self.curves.len() as f64 * vec.x - curve_idx as f64;
        self.curves[curve_idx].parametric(&Vector1::new(curve_t))
    }
}
