use super::parametric_form::DifferentialParametricForm;
use crate::math::{
    functions::{SurfacePointL2DistanceSquared, SurfaceSurfaceL2DistanceSquared},
    gradient_descent::GradientDescent,
    newtons_algorithm::NewtonsAlgorithm,
    utils::point_avg,
};
use nalgebra::{point, vector, Point2, Point3};
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone, Copy)]
pub struct IntersectionPoint {
    pub surface_0: Point2<f64>,
    pub surface_1: Point2<f64>,
    pub point: Point3<f64>,
}

#[derive(Debug)]
pub struct Intersection {
    pub wrapped: bool,
    pub points: Vec<IntersectionPoint>,
}

pub struct IntersectionFinder<'f> {
    surface_0: &'f dyn DifferentialParametricForm<2, 3>,
    surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    pub guide_point: Option<Point3<f64>>,
    pub step: f64,
}

impl<'f> IntersectionFinder<'f> {
    const STOCHASTIC_FIRST_POINT_TRIES: usize = 10;

    pub fn new(
        surface_0: &'f dyn DifferentialParametricForm<2, 3>,
        surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    ) -> Self {
        Self {
            surface_0,
            surface_1,
            guide_point: None,
            step: 0.0001,
        }
    }

    pub fn find(&self) -> Option<Intersection> {
        let first_point = self.find_first_point()?;

        Some(Intersection {
            wrapped: false,
            points: vec![first_point],
        })
    }

    fn find_first_point(&self) -> Option<IntersectionPoint> {
        if let Some(guide) = self.guide_point {
            self.find_common_point_with_guide(guide)
        } else {
            self.find_common_point_stochastic()
        }
    }

    fn find_common_point_with_guide(&self, guide: Point3<f64>) -> Option<IntersectionPoint> {
        let projection_0 = self.find_point_projection(self.surface_0, guide);
        let projection_1 = self.find_point_projection(self.surface_1, guide);

        self.find_common_surface_point(projection_0, projection_1)
    }

    fn find_common_point_stochastic(&self) -> Option<IntersectionPoint> {
        let bounds = self.surface_0.bounds();
        let mut rng = rand::thread_rng();
        let u_distribution = Uniform::new_inclusive(bounds.x.0, bounds.x.1);
        let v_distribution = Uniform::new_inclusive(bounds.y.0, bounds.y.1);

        for _ in 0..Self::STOCHASTIC_FIRST_POINT_TRIES {
            let point_0 = point![
                u_distribution.sample(&mut rng),
                v_distribution.sample(&mut rng)
            ];

            let surface_0_point = self.surface_0.value(&point_0.coords);
            let point_1 = self.find_point_projection(self.surface_1, surface_0_point);

            let common_point = self.find_common_surface_point(point_0, point_1);

            if common_point.is_some() {
                return common_point;
            }
        }

        None
    }

    fn find_point_projection(
        &self,
        surface: &dyn DifferentialParametricForm<2, 3>,
        point: Point3<f64>,
    ) -> Point2<f64> {
        let surface_point_distance = SurfacePointL2DistanceSquared::new(surface, point);

        let mut gradient_descent = GradientDescent::new(&surface_point_distance);
        gradient_descent.step = self.step;
        gradient_descent.calculate().into()
    }

    fn find_common_surface_point(
        &self,
        start_0: Point2<f64>,
        start_1: Point2<f64>,
    ) -> Option<IntersectionPoint> {
        let surface_surface_distance =
            SurfaceSurfaceL2DistanceSquared::new(self.surface_0, self.surface_1);

        let mut gradient_descent = GradientDescent::new(&surface_surface_distance);
        gradient_descent.step = self.step;
        gradient_descent.starting_point = vector![start_0.x, start_0.y, start_1.x, start_1.y];

        let minimum = gradient_descent.calculate();
        let surface_0_minimum = vector![minimum.x, minimum.y];
        let surface_1_minimum = vector![minimum.z, minimum.w];
        let surface_0_val = self.surface_0.value(&surface_0_minimum);
        let surface_1_val = self.surface_1.value(&surface_1_minimum);

        if (surface_0_val - surface_1_val).norm() > self.step {
            return None;
        }

        let midpoint = point_avg(surface_0_val, surface_1_val);
        Some(IntersectionPoint {
            surface_0: surface_0_minimum.into(),
            surface_1: surface_1_minimum.into(),
            point: midpoint,
        })
    }
}
