use super::parametric_form::ParametricForm;
use crate::{
    math::{
        bernstein_polynomial::BernsteinPolynomial, bspline::CubicBSpline, utils::point_64_to_32,
    },
    utils::transpose_vector,
};
use nalgebra::{Point3, Vector1};

#[derive(Clone, Debug)]
pub struct BezierCurve {
    x_t: BernsteinPolynomial<f64>,
    y_t: BernsteinPolynomial<f64>,
    z_t: BernsteinPolynomial<f64>,
}

impl BezierCurve {
    pub fn through_points(points: &[Point3<f64>]) -> Self {
        Self {
            x_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.x).collect()),
            y_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.y).collect()),
            z_t: BernsteinPolynomial::with_coefficients(points.iter().map(|p| p.z).collect()),
        }
    }

    pub fn points(&self) -> Vec<Point3<f64>> {
        self.x_t
            .coeffs
            .iter()
            .zip(self.y_t.coeffs.iter())
            .zip(self.z_t.coeffs.iter())
            .map(|((&x, &y), &z)| Point3::new(x, y, z))
            .collect()
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
            curves.push(BezierCurve::through_points(&[
                points[i * 3],
                points[i * 3 + 1],
                points[i * 3 + 2],
                points[i * 3 + 3],
            ]));
        }

        let i = curve_count - 1;
        curves.push(match (points.len() - 1) % 3 {
            0 => BezierCurve::through_points(&[points[i * 3]]),
            1 => BezierCurve::through_points(&[points[i * 3], points[i * 3 + 1]]),
            2 => {
                BezierCurve::through_points(&[points[i * 3], points[i * 3 + 1], points[i * 3 + 2]])
            }

            _ => panic!("Invalid remainder"),
        });

        Self { curves }
    }

    pub fn segments(&self) -> &[BezierCurve] {
        &self.curves
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

#[derive(Clone, Debug)]
pub struct PointsGrid {
    pub points: Vec<Vec<Point3<f64>>>,
}

impl PointsGrid {
    pub fn new(points: Vec<Vec<Point3<f64>>>) -> Self {
        Self { points }
    }

    pub fn u_points(&self) -> usize {
        self.points.len()
    }

    pub fn v_points(&self) -> usize {
        self.points.first().map_or(0, |u| u.len())
    }

    pub fn point(&self, u: usize, v: usize) -> Point3<f64> {
        self.points[u][v]
    }

    pub fn flat_points(&self) -> Vec<Point3<f32>> {
        self.points
            .iter()
            .flatten()
            .copied()
            .map(point_64_to_32)
            .collect()
    }

    pub fn flat_idx(&self, u: usize, v: usize) -> usize {
        u * self.v_points() + v
    }
}

#[derive(Clone, Debug)]
pub struct BezierSurface {
    grid: PointsGrid,
}

impl BezierSurface {
    pub fn new(points: Vec<Vec<Point3<f64>>>) -> Self {
        assert!(points.is_empty() || (points.len() - 1) % 3 == 0 && (points[0].len() - 1) % 3 == 0);
        Self {
            grid: PointsGrid::new(points),
        }
    }

    pub fn u_patches(&self) -> usize {
        // Needs to be valid only for 0, 4, and 4 + 3k
        (self.grid.u_points() + 1) / 3
    }

    pub fn v_patches(&self) -> usize {
        self.grid.points.first().map_or(0, |u| (u.len() + 1) / 3)
    }

    pub fn patch_point(
        &self,
        patch_u: usize,
        patch_v: usize,
        point_u: usize,
        point_v: usize,
    ) -> Point3<f64> {
        self.grid.points[patch_u * 3 + point_u][patch_v * 3 + point_v]
    }

    pub fn grid(&self) -> &PointsGrid {
        &self.grid
    }
}

pub fn deboor_surface_to_bernstein(deboor_points: Vec<Vec<Point3<f64>>>) -> Vec<Vec<Point3<f64>>> {
    let bernstein_u_splines: Vec<_> = deboor_points
        .into_iter()
        .map(|points| BezierBSpline::through_points(points).bernstein_points())
        .collect();

    let transpose = transpose_vector(&bernstein_u_splines);

    transpose_vector(
        &transpose
            .into_iter()
            .map(|points| BezierBSpline::through_points(points).bernstein_points())
            .collect(),
    )
}
