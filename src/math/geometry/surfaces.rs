use super::{
    bezier::{deboor_surface_to_bernstein, BezierCurve, BezierSurface},
    parametric_form::{DifferentialParametricForm, ParametricForm},
};
use nalgebra::{Matrix3x2, Point3, Vector1, Vector2};

type ControlPatch = [[Point3<f64>; 4]; 4];

#[derive(Clone, Debug)]
pub struct PatchC0(pub ControlPatch);

impl PatchC0 {
    pub fn new(patch: ControlPatch) -> Self {
        Self(patch)
    }
}

impl DifferentialParametricForm<2, 3> for PatchC0 {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new((0.0, 1.0), (0.0, 1.0))
    }

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        let bezier_points: Vec<_> = self
            .0
            .iter()
            .map(|patch_row| {
                BezierCurve::through_points(patch_row).parametric(&Vector1::new(vec.y))
            })
            .collect();

        BezierCurve::through_points(&bezier_points).parametric(&Vector1::new(vec.x))
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceC0(Vec<Vec<PatchC0>>);

impl SurfaceC0 {
    pub fn null() -> Self {
        Self(Vec::new())
    }

    pub fn patch(&self, u: usize, v: usize) -> PatchC0 {
        self.0[u][v].clone()
    }

    pub fn from_patches(patches: Vec<Vec<PatchC0>>) -> Self {
        Self(patches)
    }

    pub fn from_points(bezier_points: Vec<Vec<Point3<f64>>>) -> Self {
        let surface = BezierSurface::new(bezier_points);
        Self::from_bezier_surface(surface)
    }

    pub fn from_bezier_surface(surface: BezierSurface) -> Self {
        let mut patches = Vec::new();

        for patch_u in 0..surface.u_patches() {
            patches.push(Vec::new());

            for patch_v in 0..surface.v_patches() {
                patches.last_mut().unwrap().push(PatchC0::new([
                    [
                        surface.patch_point(patch_u, patch_v, 0, 0),
                        surface.patch_point(patch_u, patch_v, 0, 1),
                        surface.patch_point(patch_u, patch_v, 0, 2),
                        surface.patch_point(patch_u, patch_v, 0, 3),
                    ],
                    [
                        surface.patch_point(patch_u, patch_v, 1, 0),
                        surface.patch_point(patch_u, patch_v, 1, 1),
                        surface.patch_point(patch_u, patch_v, 1, 2),
                        surface.patch_point(patch_u, patch_v, 1, 3),
                    ],
                    [
                        surface.patch_point(patch_u, patch_v, 2, 0),
                        surface.patch_point(patch_u, patch_v, 2, 1),
                        surface.patch_point(patch_u, patch_v, 2, 2),
                        surface.patch_point(patch_u, patch_v, 2, 3),
                    ],
                    [
                        surface.patch_point(patch_u, patch_v, 3, 0),
                        surface.patch_point(patch_u, patch_v, 3, 1),
                        surface.patch_point(patch_u, patch_v, 3, 2),
                        surface.patch_point(patch_u, patch_v, 3, 3),
                    ],
                ]))
            }
        }

        Self::from_patches(patches)
    }

    fn u_patches(&self) -> usize {
        self.0.len()
    }

    fn v_patches(&self) -> usize {
        if self.0.len() == 0 {
            0
        } else {
            self.0[0].len()
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
}

impl DifferentialParametricForm<2, 3> for SurfaceC0 {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        Vector2::new((0.0, 1.0), (0.0, 1.0))
    }

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        let u_patch = Self::patch_idx(vec.x, self.u_patches());
        let v_patch = Self::patch_idx(vec.y, self.v_patches());

        let u = Self::patch_parameter(vec.x, self.u_patches());
        let v = Self::patch_parameter(vec.y, self.v_patches());

        DifferentialParametricForm::parametric(&self.0[u_patch][v_patch], &Vector2::new(u, v))
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceC2(SurfaceC0);

impl SurfaceC2 {
    pub fn from_points(deboor_points: Vec<Vec<Point3<f64>>>) -> Self {
        let surface_c0 = SurfaceC0::from_points(deboor_surface_to_bernstein(deboor_points));
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

    fn parametric(&self, vec: &Vector2<f64>) -> Point3<f64> {
        DifferentialParametricForm::parametric(&self.0, vec)
    }

    fn jacobian(&self, vec: &Vector2<f64>) -> Matrix3x2<f64> {
        self.0.jacobian(vec)
    }
}
