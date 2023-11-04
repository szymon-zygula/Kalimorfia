use super::{
    bezier::{deboor_surface_to_bernstein, BezierCurve, BezierSurface},
    parametric_form::{DifferentialParametricForm, ParametricForm},
};
use itertools::Itertools;
use nalgebra::{matrix, vector, Matrix3x2, Point3, Vector1, Vector2, Vector3};

#[derive(Clone, Debug)]
pub struct XZPlane {
    size: Vector2<f64>,
    origin: Point3<f64>,
}

impl XZPlane {
    pub fn new(origin: Point3<f64>, size: Vector2<f64>) -> Self {
        Self { size, origin }
    }
}

impl DifferentialParametricForm<2, 3> for XZPlane {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        vector![(0.0, self.size.x), (0.0, self.size.y)]
    }

    fn wrapped(&self, _dim: usize) -> bool {
        false
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        self.origin + vector![vec.x, 0.0, vec.y]
    }

    fn jacobian(&self, _vec: &Vector2<f64>) -> Matrix3x2<f64> {
        matrix![
            1.0, 0.0;
            0.0, 0.0;
            0.0, 1.0;
        ]
    }
}

#[derive(Clone, Debug)]
pub struct BezierPatch {
    control_points: Vec<Vec<Point3<f64>>>,
    u_derivative: Option<Box<BezierPatch>>,
    v_derivative: Option<Box<BezierPatch>>,
}

impl BezierPatch {
    pub fn new(
        control_points: Vec<Vec<Point3<f64>>>,
        derivatives: bool,
        second_derivatives: bool,
    ) -> Self {
        let (u_derivative, v_derivative) = if derivatives {
            let u_degree = control_points.len() - 1;
            let v_degree = control_points[0].len() - 1;

            let v_control_points = control_points
                .iter()
                .map(|row| {
                    row.iter()
                        .tuple_windows()
                        .map(|(p0, p1)| ((p1 - p0) * v_degree as f64).into())
                        .collect()
                })
                .collect();

            let u_control_points = control_points
                .iter()
                .tuple_windows()
                .map(|(row0, row1)| {
                    row0.iter()
                        .zip(row1.iter())
                        .map(|(p0, p1)| ((p1 - p0) * u_degree as f64).into())
                        .collect()
                })
                .collect();

            (
                Some(Box::new(BezierPatch::new(
                    u_control_points,
                    second_derivatives,
                    false,
                ))),
                Some(Box::new(BezierPatch::new(
                    v_control_points,
                    second_derivatives,
                    false,
                ))),
            )
        } else {
            (None, None)
        };

        Self {
            control_points,
            u_derivative,
            v_derivative,
        }
    }
}

impl DifferentialParametricForm<2, 3> for BezierPatch {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new((0.0, 1.0), (0.0, 1.0))
    }

    fn wrapped(&self, _dim: usize) -> bool {
        false
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        let bezier_points: Vec<_> = self
            .control_points
            .iter()
            .map(|patch_row| BezierCurve::through_points(patch_row).value(&Vector1::new(vec.y)))
            .collect();

        BezierCurve::through_points(&bezier_points).value(&Vector1::new(vec.x))
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        Matrix3x2::from_columns(&[
            DifferentialParametricForm::value(&**self.u_derivative.as_ref().unwrap(), vec).coords,
            DifferentialParametricForm::value(&**self.v_derivative.as_ref().unwrap(), vec).coords,
        ])
    }

    fn hessian(&self, vec: &Vector2<f64>, var_0: usize, var_1: usize) -> Vector3<f64> {
        let derivative_0 = match var_0 {
            0 => self.u_derivative.as_ref().unwrap(),
            1 => self.v_derivative.as_ref().unwrap(),
            _ => panic!("Bezier patches are 2-dimensional"),
        };

        DifferentialParametricForm::value(
            match var_1 {
                0 => derivative_0.u_derivative.as_ref().unwrap().as_ref(),
                1 => derivative_0.v_derivative.as_ref().unwrap().as_ref(),
                _ => panic!("Bezier patches are 2-dimensional"),
            },
            vec,
        )
        .coords
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceC0 {
    patches: Vec<Vec<BezierPatch>>,
    u_wrap: bool,
    v_wrap: bool,
}

impl SurfaceC0 {
    pub fn null() -> Self {
        Self {
            patches: Vec::new(),
            u_wrap: false,
            v_wrap: false,
        }
    }

    pub fn from_patches(patches: Vec<Vec<BezierPatch>>, u_wrap: bool, v_wrap: bool) -> Self {
        Self {
            patches,
            u_wrap,
            v_wrap,
        }
    }

    pub fn from_points(bezier_points: Vec<Vec<Point3<f64>>>, u_wrap: bool, v_wrap: bool) -> Self {
        let surface = BezierSurface::new(bezier_points);
        Self::from_bezier_surface(surface, u_wrap, v_wrap)
    }

    pub fn from_bezier_surface(surface: BezierSurface, u_wrap: bool, v_wrap: bool) -> Self {
        let mut patches = Vec::new();

        for patch_u in 0..surface.u_patches() {
            patches.push(Vec::new());

            for patch_v in 0..surface.v_patches() {
                patches.last_mut().unwrap().push(BezierPatch::new(
                    vec![
                        vec![
                            surface.patch_point(patch_u, patch_v, 0, 0),
                            surface.patch_point(patch_u, patch_v, 0, 1),
                            surface.patch_point(patch_u, patch_v, 0, 2),
                            surface.patch_point(patch_u, patch_v, 0, 3),
                        ],
                        vec![
                            surface.patch_point(patch_u, patch_v, 1, 0),
                            surface.patch_point(patch_u, patch_v, 1, 1),
                            surface.patch_point(patch_u, patch_v, 1, 2),
                            surface.patch_point(patch_u, patch_v, 1, 3),
                        ],
                        vec![
                            surface.patch_point(patch_u, patch_v, 2, 0),
                            surface.patch_point(patch_u, patch_v, 2, 1),
                            surface.patch_point(patch_u, patch_v, 2, 2),
                            surface.patch_point(patch_u, patch_v, 2, 3),
                        ],
                        vec![
                            surface.patch_point(patch_u, patch_v, 3, 0),
                            surface.patch_point(patch_u, patch_v, 3, 1),
                            surface.patch_point(patch_u, patch_v, 3, 2),
                            surface.patch_point(patch_u, patch_v, 3, 3),
                        ],
                    ],
                    true,
                    true,
                ))
            }
        }

        Self::from_patches(patches, u_wrap, v_wrap)
    }

    pub fn patch(&self, u: usize, v: usize) -> BezierPatch {
        self.patches[u][v].clone()
    }

    fn u_patches(&self) -> usize {
        self.patches.len()
    }

    fn v_patches(&self) -> usize {
        if self.patches.is_empty() {
            0
        } else {
            self.patches[0].len()
        }
    }

    /// Returns u or v index of a patch with `val` as u or v parameter value of the surface
    fn patch_idx(val: f64, count: usize) -> usize {
        if val == 1.0 {
            count - 1
        } else {
            (val * count as f64).floor() as usize
        }
    }

    /// Returns u or v parameter of a patch with `val` as u or v parameter of the surface
    fn patch_parameter(val: f64, count: usize) -> f64 {
        if val == 1.0 {
            val
        } else {
            (val * count as f64).fract()
        }
    }

    fn patch_for_param(&self, vec: &Vector2<f64>) -> &BezierPatch {
        let u_patch = Self::patch_idx(vec.x, self.u_patches());
        let v_patch = Self::patch_idx(vec.y, self.v_patches());
        &self.patches[u_patch][v_patch]
    }

    fn param_for_param(&self, vec: &Vector2<f64>) -> Vector2<f64> {
        let u = Self::patch_parameter(vec.x, self.u_patches());
        let v = Self::patch_parameter(vec.y, self.v_patches());

        Vector2::new(u, v)
    }
}

impl DifferentialParametricForm<2, 3> for SurfaceC0 {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new((0.0, 1.0), (0.0, 1.0))
    }

    fn wrapped(&self, dim: usize) -> bool {
        match dim {
            0 => self.u_wrap,
            1 => self.v_wrap,
            _ => false,
        }
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        DifferentialParametricForm::value(self.patch_for_param(vec), &self.param_for_param(vec))
    }

    // The first and second derivatives don't exist at patch borders, but it doesn't matter...
    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        let jacobian = DifferentialParametricForm::jacobian(
            self.patch_for_param(vec),
            &self.param_for_param(vec),
        );

        let scaled_u = jacobian.fixed_view::<3, 1>(0, 0) * self.u_patches() as f64;
        let scaled_v = jacobian.fixed_view::<3, 1>(0, 1) * self.v_patches() as f64;

        Matrix3x2::from_columns(&[scaled_u, scaled_v])
    }

    fn hessian(&self, vec: &Vector2<f64>, var_0: usize, var_1: usize) -> Vector3<f64> {
        let hessian = DifferentialParametricForm::hessian(
            self.patch_for_param(vec),
            &self.param_for_param(vec),
            var_0,
            var_1,
        );

        let multiplier_0 = match var_0 {
            0 => self.u_patches(),
            1 => self.v_patches(),
            _ => panic!("SurfaceC0 is 2-dimensional"),
        } as f64;

        let multiplier_1 = match var_1 {
            0 => self.u_patches(),
            1 => self.v_patches(),
            _ => panic!("SurfaceC0 is 2-dimensional"),
        } as f64;

        hessian * multiplier_0 * multiplier_1
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceC2(SurfaceC0);

impl SurfaceC2 {
    pub fn from_points(deboor_points: Vec<Vec<Point3<f64>>>, u_wrap: bool, v_wrap: bool) -> Self {
        let surface_c0 =
            SurfaceC0::from_points(deboor_surface_to_bernstein(deboor_points), u_wrap, v_wrap);
        Self(surface_c0)
    }

    pub fn null() -> Self {
        Self(SurfaceC0::null())
    }
}

impl DifferentialParametricForm<2, 3> for SurfaceC2 {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new((0.0, 1.0), (0.0, 1.0))
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.0.wrapped(dim)
    }

    fn value(&self, vec: &Vector2<f64>) -> Point3<f64> {
        DifferentialParametricForm::value(&self.0, vec)
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        self.0.jacobian(vec)
    }

    fn hessian(&self, vec: &Vector2<f64>, var_0: usize, var_1: usize) -> Vector3<f64> {
        self.0.hessian(vec, var_0, var_1)
    }
}

pub struct NormalField<'a>(&'a dyn DifferentialParametricForm<2, 3>);

impl<'a> NormalField<'a> {
    pub fn new(surface: &'a dyn DifferentialParametricForm<2, 3>) -> Self {
        Self(surface)
    }

    pub fn anormal(&self, vec: &Vector2<f64>) -> Vector3<f64> {
        let jacobian = self.0.jacobian(vec);

        jacobian
            .fixed_columns::<1>(0)
            .cross(&jacobian.fixed_columns::<1>(1))
    }
}

impl<'a> DifferentialParametricForm<2, 3> for NormalField<'a> {
    fn bounds(&self) -> nalgebra::SVector<(f64, f64), 2> {
        self.0.bounds()
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.0.wrapped(dim)
    }

    fn value(&self, vec: &nalgebra::SVector<f64, 2>) -> Point3<f64> {
        self.anormal(vec).normalize().into()
    }

    fn jacobian(&self, vec: &nalgebra::SVector<f64, 2>) -> Matrix3x2<f64> {
        let jacobian = self.0.jacobian(vec);
        let diff_u = jacobian.fixed_columns::<1>(0);
        let diff_v = jacobian.fixed_columns::<1>(1);
        let diff_uu = self.0.hessian(vec, 0, 0);
        let diff_vu = self.0.hessian(vec, 1, 0);
        let diff_vv = self.0.hessian(vec, 1, 1);

        let anorm_diff_u = diff_uu.cross(&diff_v) + diff_u.cross(&diff_vu);
        let anorm_diff_v = diff_vu.cross(&diff_v) + diff_u.cross(&diff_vv);

        let anormal = self.anormal(vec);
        let norm = anormal.norm();

        Matrix3x2::from_columns(&[
            anorm_diff_u / norm
                - anormal * Vector3::dot(&anormal, &anorm_diff_u) / (norm * norm * norm),
            anorm_diff_v / norm
                - anormal * Vector3::dot(&anormal, &anorm_diff_v) / (norm * norm * norm),
        ])
    }
}

pub struct ShiftedSurface<'a> {
    pub surface: &'a dyn DifferentialParametricForm<2, 3>,
    pub distance: f64,
}

impl<'a> ShiftedSurface<'a> {
    pub fn new(surface: &'a dyn DifferentialParametricForm<2, 3>, distance: f64) -> Self {
        Self { surface, distance }
    }
}

impl<'a> DifferentialParametricForm<2, 3> for ShiftedSurface<'a> {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        self.surface.bounds()
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.surface.wrapped(dim)
    }

    fn value(&self, vec: &Vector2<f64>) -> nalgebra::Point<f64, 3> {
        let normal_field = NormalField::new(self.surface);
        self.surface.value(vec)
            + DifferentialParametricForm::value(&normal_field, vec).coords * self.distance
    }

    fn jacobian(&self, vec: &nalgebra::SVector<f64, 2>) -> nalgebra::SMatrix<f64, 3, 2> {
        let normal_field = NormalField::new(self.surface);
        self.surface.jacobian(vec) + self.distance * normal_field.jacobian(vec)
    }
}
