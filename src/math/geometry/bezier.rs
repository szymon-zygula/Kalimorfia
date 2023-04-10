use super::parametric_form::ParametricForm;
use crate::math::{bernstein_polynomial::BernsteinPolynomial, bspline::CubicBSpline};
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
pub struct BezierCubicSplineC0 {
    curves: Vec<BezierCurve>,
}

impl BezierCubicSplineC0 {
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

impl ParametricForm<1, 3> for BezierCubicSplineC0 {
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

#[derive(Clone, Debug)]
pub struct BezierBSpline {
    x_t: CubicBSpline,
    y_t: CubicBSpline,
    z_t: CubicBSpline,
}

impl BezierBSpline {
    pub fn through_points(points: Vec<Point3<f64>>) -> Self {
        assert!(points.len() >= 4);

        Self {
            x_t: CubicBSpline::with_coefficients(points.iter().map(|p| p.x).collect()),
            y_t: CubicBSpline::with_coefficients(points.iter().map(|p| p.y).collect()),
            z_t: CubicBSpline::with_coefficients(points.iter().map(|p| p.z).collect()),
        }
    }

    pub fn modify_bernstein(&self, point_idx: usize, val: Point3<f64>) -> Self {
        Self {
            x_t: self.x_t.modify_bernstein(point_idx, val.x),
            y_t: self.y_t.modify_bernstein(point_idx, val.y),
            z_t: self.z_t.modify_bernstein(point_idx, val.z),
        }
    }

    pub fn bernstein_points(&self) -> Vec<Point3<f64>> {
        let bernstein_x = self.x_t.bernstein_values();
        let bernstein_y = self.y_t.bernstein_values();
        let bernstein_z = self.z_t.bernstein_values();
        let mut bernstein = Vec::new();

        for i in 0..bernstein_x.len() {
            bernstein.push(Point3::new(bernstein_x[i], bernstein_y[i], bernstein_z[i]));
        }

        bernstein
    }

    pub fn deboor_points(&self) -> Vec<Point3<f64>> {
        let deboor_x = self.x_t.deboor_points();
        let deboor_y = self.y_t.deboor_points();
        let deboor_z = self.z_t.deboor_points();
        let mut deboor = Vec::new();

        for i in 0..deboor_x.len() {
            deboor.push(Point3::new(deboor_x[i], deboor_y[i], deboor_z[i]))
        }

        deboor
    }

    fn points_f32(points: &[Point3<f64>]) -> Vec<Point3<f32>> {
        points
            .iter()
            .map(|p| Point3::new(p.x as f32, p.y as f32, p.z as f32))
            .collect()
    }

    pub fn bernstein_points_f32(&self) -> Vec<Point3<f32>> {
        Self::points_f32(&self.bernstein_points())
    }

    pub fn deboor_points_f32(&self) -> Vec<Point3<f32>> {
        Self::points_f32(&self.deboor_points())
    }
}

impl ParametricForm<1, 3> for BezierBSpline {
    const PARAMETER_BOUNDS: Vector1<(f64, f64)> = Vector1::new((0.0, 1.0));

    fn parametric(&self, vec: &Vector1<f64>) -> Point3<f64> {
        Point3::new(
            self.x_t.value(vec.x),
            self.y_t.value(vec.x),
            self.z_t.value(vec.x),
        )
    }
}
