use crate::math::geometry::parametric_form::DifferentialParametricForm;
use nalgebra::{vector, Matrix3x4, Point3, SVector, Vector2, Vector4};

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
        self.parametric(x).x
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
        let val_0 = self.surface_0.parametric(&Vector2::new(x.x, x.y));
        let val_1 = self.surface_1.parametric(&Vector2::new(x.z, x.w));

        (val_0 - val_1).norm_squared()
    }

    fn grad(&self, x: &Vector4<f64>) -> Vector4<f64> {
        let arg_0 = Vector2::new(x.x, x.y);
        let arg_1 = Vector2::new(x.z, x.w);

        let jacobian_0 = self.surface_0.jacobian(&arg_0);
        let jacobian_1 = self.surface_1.jacobian(&arg_1);

        let combined_jacobian = Matrix3x4::from_columns(&[
            jacobian_0.fixed_view::<3, 1>(0, 0),
            jacobian_0.fixed_view::<3, 1>(0, 1),
            jacobian_1.fixed_view::<3, 1>(0, 0),
            jacobian_1.fixed_view::<3, 1>(0, 1),
        ]);

        2.0 * combined_jacobian.transpose()
            * (self.surface_0.parametric(&arg_0) - self.surface_1.parametric(&arg_1))
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
        (self.surface.parametric(x) - self.point).norm_squared()
    }

    fn grad(&self, x: &Vector2<f64>) -> Vector2<f64> {
        2.0 * self.surface.jacobian(x).transpose() * (self.surface.parametric(x) - self.point)
    }
}
