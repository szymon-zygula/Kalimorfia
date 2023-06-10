use crate::math::geometry::parametric_form::DifferentialParametricForm;
use nalgebra::{
    matrix, point, vector, Matrix3x4, Matrix4, Point3, Point4, SVector, Vector2, Vector3, Vector4,
};

use super::utils::point_avg;

pub trait DifferentiableScalarFunction<const DIM: usize> {
    fn bounds(&self) -> SVector<(f64, f64), DIM>;
    fn wrapped(&self, dim: usize) -> bool;

    fn val(&self, x: &SVector<f64, DIM>) -> f64;
    fn grad(&self, x: &SVector<f64, DIM>) -> SVector<f64, DIM>;
}

impl<const DIM: usize, T> DifferentiableScalarFunction<DIM> for T
where
    T: DifferentialParametricForm<DIM, 1>,
{
    fn bounds(&self) -> SVector<(f64, f64), DIM> {
        self.bounds()
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.wrapped(dim)
    }

    fn val(&self, x: &SVector<f64, DIM>) -> f64 {
        self.value(x).x
    }

    fn grad(&self, x: &SVector<f64, DIM>) -> SVector<f64, DIM> {
        self.jacobian(x).transpose()
    }
}

pub struct SurfaceSurfaceL2DistanceSquared<'f> {
    surface_0: &'f dyn DifferentialParametricForm<2, 3>,
    surface_1: &'f dyn DifferentialParametricForm<2, 3>,
}

impl<'f> SurfaceSurfaceL2DistanceSquared<'f> {
    pub fn new(
        surface_0: &'f dyn DifferentialParametricForm<2, 3>,
        surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    ) -> Self {
        Self {
            surface_0,
            surface_1,
        }
    }
}

impl<'f> DifferentiableScalarFunction<4> for SurfaceSurfaceL2DistanceSquared<'f> {
    fn bounds(&self) -> Vector4<(f64, f64)> {
        let bounds_0 = self.surface_0.bounds();
        let bounds_1 = self.surface_1.bounds();

        vector![bounds_0.x, bounds_0.y, bounds_1.x, bounds_1.y]
    }

    fn wrapped(&self, dim: usize) -> bool {
        match dim {
            0 | 1 => self.surface_0.wrapped(dim),
            2 | 3 => self.surface_1.wrapped(dim - 2),
            _ => false,
        }
    }

    fn val(&self, x: &Vector4<f64>) -> f64 {
        let val_0 = self.surface_0.value(&vector!(x.x, x.y));
        let val_1 = self.surface_1.value(&vector!(x.z, x.w));

        (val_0 - val_1).norm_squared()
    }

    fn grad(&self, x: &Vector4<f64>) -> Vector4<f64> {
        let arg_0 = vector!(x.x, x.y);
        let arg_1 = vector!(x.z, x.w);

        let jacobian_0 = self.surface_0.jacobian(&arg_0);
        let jacobian_1 = -self.surface_1.jacobian(&arg_1);

        let combined_jacobian = Matrix3x4::from_columns(&[
            jacobian_0.fixed_view::<3, 1>(0, 0),
            jacobian_0.fixed_view::<3, 1>(0, 1),
            jacobian_1.fixed_view::<3, 1>(0, 0),
            jacobian_1.fixed_view::<3, 1>(0, 1),
        ]);

        2.0 * combined_jacobian.transpose()
            * (self.surface_0.value(&arg_0) - self.surface_1.value(&arg_1))
    }
}

pub struct SurfacePointL2DistanceSquared<'f> {
    surface: &'f dyn DifferentialParametricForm<2, 3>,
    point: Point3<f64>,
}

impl<'f> SurfacePointL2DistanceSquared<'f> {
    pub fn new(surface: &'f dyn DifferentialParametricForm<2, 3>, point: Point3<f64>) -> Self {
        Self { surface, point }
    }
}

impl<'f> DifferentiableScalarFunction<2> for SurfacePointL2DistanceSquared<'f> {
    fn bounds(&self) -> Vector2<(f64, f64)> {
        self.surface.bounds()
    }

    fn wrapped(&self, dim: usize) -> bool {
        self.surface.wrapped(dim)
    }
    fn val(&self, x: &Vector2<f64>) -> f64 {
        (self.surface.value(x) - self.point).norm_squared()
    }

    fn grad(&self, x: &Vector2<f64>) -> Vector2<f64> {
        2.0 * self.surface.jacobian(x).transpose() * (self.surface.value(x) - self.point)
    }
}

pub struct IntersectionStepFunction<'f> {
    surface_0: &'f dyn DifferentialParametricForm<2, 3>,
    surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    common_point: Point3<f64>,
    direction: Vector3<f64>,
    step: f64,
}

impl<'f> IntersectionStepFunction<'f> {
    pub fn new(
        surface_0: &'f dyn DifferentialParametricForm<2, 3>,
        surface_1: &'f dyn DifferentialParametricForm<2, 3>,
        common_point: Point3<f64>,
        direction: Vector3<f64>,
        step: f64,
    ) -> Self {
        Self {
            surface_0,
            surface_1,
            common_point,
            direction,
            step,
        }
    }
}

impl<'f> DifferentialParametricForm<4, 4> for IntersectionStepFunction<'f> {
    fn bounds(&self) -> SVector<(f64, f64), 4> {
        let bounds_0 = self.surface_0.bounds();
        let bounds_1 = self.surface_1.bounds();

        vector![bounds_0.x, bounds_0.y, bounds_1.x, bounds_1.y]
    }

    fn wrapped(&self, dim: usize) -> bool {
        match dim {
            0 | 1 => self.surface_0.wrapped(dim),
            2 | 3 => self.surface_1.wrapped(dim - 2),
            _ => false,
        }
    }

    fn value(&self, vec: &SVector<f64, 4>) -> Point4<f64> {
        let surface_0_val = self.surface_0.value(&vector![vec.x, vec.y]);
        let surface_1_val = self.surface_1.value(&vector![vec.z, vec.w]);
        let surface_diff = surface_0_val - surface_1_val;

        let midpoint = point_avg(surface_0_val, surface_1_val);
        let displacement_projection_length =
            Vector3::dot(&(midpoint - self.common_point), &self.direction) - self.step;

        point![
            surface_diff.x,
            surface_diff.y,
            surface_diff.z,
            displacement_projection_length
        ]
    }

    fn jacobian(&self, vec: &SVector<f64, 4>) -> Matrix4<f64> {
        let surface_0_jacobian = self.surface_0.jacobian(&vector![vec.x, vec.y]);
        let surface_1_jacobian = self.surface_1.jacobian(&vector![vec.z, vec.w]);
        let surface_1_jacobian_neg = -self.surface_1.jacobian(&vector![vec.z, vec.w]);

        let combined_add_jacobian = Matrix3x4::from_columns(&[
            surface_0_jacobian.fixed_view::<3, 1>(0, 0),
            surface_0_jacobian.fixed_view::<3, 1>(0, 1),
            surface_1_jacobian.fixed_view::<3, 1>(0, 0),
            surface_1_jacobian.fixed_view::<3, 1>(0, 1),
        ]);

        let combined_sub_jacobian = Matrix3x4::from_columns(&[
            surface_0_jacobian.fixed_view::<3, 1>(0, 0),
            surface_0_jacobian.fixed_view::<3, 1>(0, 1),
            surface_1_jacobian_neg.fixed_view::<3, 1>(0, 0),
            surface_1_jacobian_neg.fixed_view::<3, 1>(0, 1),
        ]);

        let projection_jacobian = 0.5 * self.direction.transpose() * combined_add_jacobian;

        let csj = &combined_sub_jacobian;
        let prj = &projection_jacobian;

        matrix![
            csj[(0, 0)], csj[(0, 1)], csj[(0, 2)], csj[(0, 3)];
            csj[(1, 0)], csj[(1, 1)], csj[(1, 2)], csj[(1, 3)];
            csj[(2, 0)], csj[(2, 1)], csj[(2, 2)], csj[(2, 3)];
            prj[(0, 0)], prj[(0, 1)], prj[(0, 2)], prj[(0, 3)];
        ]
    }
}
