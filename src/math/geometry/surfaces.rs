use super::{
    bezier::{deboor_surface_to_bernstein, BezierCurve, BezierSurface},
    parametric_form::{DifferentialParametricForm, ParametricForm},
};
use nalgebra::{Matrix3x2, Point3, Vector1, Vector2};

#[derive(Clone, Debug)]
pub struct PatchC0 {
    control_points: Vec<Vec<Point3<f64>>>,
    u_derivative: Option<Box<PatchC0>>,
    v_derivative: Option<Box<PatchC0>>,
}

impl PatchC0 {
    pub fn new(control_points: Vec<Vec<Point3<f64>>>, derivatives: bool) -> Self {
        let (u_derivative, v_derivative) = if derivatives {
            let u_degree = control_points.len() - 1;
            let v_degree = control_points[0].len() - 1;

            let mut u_control_points = Vec::new();

            for i in 0..u_degree {
                u_control_points.push(Vec::new());
                for j in 0..=v_degree {
                    u_control_points.last_mut().unwrap().push(Point3::from(
                        (control_points[i + 1][j] - control_points[i][j]) * (u_degree + 1) as f64,
                    ))
                }
            }

            let mut v_control_points = Vec::new();

            for i in 0..=u_degree {
                v_control_points.push(Vec::new());
                for j in 0..v_degree {
                    v_control_points.last_mut().unwrap().push(Point3::from(
                        (control_points[i][j + 1] - control_points[i][j]) * (v_degree + 1) as f64,
                    ))
                }
            }

            (
                Some(Box::new(PatchC0::new(u_control_points, false))),
                Some(Box::new(PatchC0::new(v_control_points, false))),
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

impl DifferentialParametricForm<2, 3> for PatchC0 {
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
            .map(|patch_row| {
                BezierCurve::through_points(patch_row).value(&Vector1::new(vec.y))
            })
            .collect();

        BezierCurve::through_points(&bezier_points).value(&Vector1::new(vec.x))
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        Matrix3x2::from_columns(&[
            DifferentialParametricForm::value(&**self.u_derivative.as_ref().unwrap(), vec)
                .coords,
            DifferentialParametricForm::value(&**self.v_derivative.as_ref().unwrap(), vec)
                .coords,
        ])
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceC0 {
    patches: Vec<Vec<PatchC0>>,
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

    pub fn from_patches(patches: Vec<Vec<PatchC0>>, u_wrap: bool, v_wrap: bool) -> Self {
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
                patches.last_mut().unwrap().push(PatchC0::new(
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
                ))
            }
        }

        Self::from_patches(patches, u_wrap, v_wrap)
    }

    pub fn patch(&self, u: usize, v: usize) -> PatchC0 {
        self.patches[u][v].clone()
    }

    fn u_patches(&self) -> usize {
        self.patches.len()
    }

    fn v_patches(&self) -> usize {
        if self.patches.len() == 0 {
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
        (val * count as f64).fract()
    }

    fn patch_for_param(&self, vec: &Vector2<f64>) -> &PatchC0 {
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
        DifferentialParametricForm::value(
            self.patch_for_param(vec),
            &self.param_for_param(vec),
        )
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        let jacobian = DifferentialParametricForm::jacobian(
            self.patch_for_param(vec),
            &self.param_for_param(vec),
        );

        let scaled_u = jacobian.fixed_view::<3, 1>(0, 0) * self.u_patches() as f64;
        let scaled_v = jacobian.fixed_view::<3, 1>(0, 1) * self.v_patches() as f64;

        Matrix3x2::from_columns(&[scaled_u, scaled_v])
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
}
