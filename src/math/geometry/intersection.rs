use super::parametric_form::{DifferentialParametricForm, WithNormals};
use crate::math::{
    functions::{
        IntersectionStepFunction, SurfacePointL2DistanceSquared, SurfaceSurfaceL2DistanceSquared,
    },
    gradient_descent::GradientDescent,
    newtons_algorithm::NewtonsAlgorithm,
    utils::point_avg,
};
use nalgebra::{vector, Point3, Vector2, Vector3};
use std::cell::RefCell;

macro_rules! tighten {
    (
        $self:ident,
        $point:ident,
        $tightening:ident,
        $surface:ident,
        $surface_other:ident
    ) => {
        let surface_val = $self.$surface.value(&$tightening.$surface);
        let surface_other_arg = $self.find_point_projection($self.$surface_other, surface_val);
        let surface_other_val = $self.$surface_other.value(&surface_other_arg);
        $tightening.$surface_other = surface_other_arg;
        $tightening.point = point_avg(surface_val, surface_other_val);

        if Vector3::metric_distance(&$tightening.point.coords, &$point.point.coords)
            < $self.intersection_step
        {
            return Some($tightening);
        }
    };
}

macro_rules! tighten_1_dim {
    ($self:ident, $point:ident, $surface:ident, $dim:expr, $surface_other:ident) => {
        if !$self.$surface.wrapped($dim) {
            let mut new_point = $point;
            new_point.$surface[$dim] = $self.$surface.bounds()[$dim].0;
            tighten!($self, $point, new_point, $surface, $surface_other);
            new_point.$surface[$dim] = $self.$surface.bounds()[$dim].1;
            tighten!($self, $point, new_point, $surface, $surface_other);
        }
    };
}

macro_rules! check_stochastic_points {
    ($self:ident, $common_point:ident) => {
        if let Some(common_point) = $common_point {
            // If this condition is not fulfilled, we've just found the same point twice
            if Vector2::metric_distance(&common_point.surface_0, &common_point.surface_1)
                >= $self.numerical_step
            {
                return Some(common_point);
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntersectionPoint {
    pub surface_0: Vector2<f64>,
    pub surface_1: Vector2<f64>,
    pub point: Point3<f64>,
}

#[derive(Debug, Clone)]
pub struct Intersection {
    pub points: Vec<IntersectionPoint>,
    pub looped: bool,
}

pub struct IntersectionFinder<'f> {
    surface_0: &'f dyn DifferentialParametricForm<2, 3>,
    surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    pub guide_point: Option<Point3<f64>>,
    pub numerical_step: f64,
    pub intersection_step: f64,
    rng: RefCell<rand::rngs::ThreadRng>,
    same: bool,
}

impl<'f> IntersectionFinder<'f> {
    const STOCHASTIC_FIRST_POINT_TRIES: usize = 500;
    const MAX_POINTS: usize = 10000;

    pub fn new(
        surface_0: &'f dyn DifferentialParametricForm<2, 3>,
        surface_1: &'f dyn DifferentialParametricForm<2, 3>,
    ) -> Self {
        Self {
            surface_0,
            surface_1,
            guide_point: None,
            numerical_step: 0.0001,
            intersection_step: 0.01,
            rng: RefCell::new(rand::thread_rng()),
            same: false,
        }
    }

    pub fn new_same(surface: &'f dyn DifferentialParametricForm<2, 3>) -> Self {
        Self {
            surface_0: surface,
            surface_1: surface,
            guide_point: None,
            numerical_step: 0.0001,
            intersection_step: 0.01,
            rng: RefCell::new(rand::thread_rng()),
            same: true,
        }
    }

    pub fn find(&self) -> Option<Intersection> {
        let first_point = self.find_first_point()?;

        let mut points = vec![first_point];

        let looped = self.push_points(&mut points, false);
        if !looped {
            points.reverse();
            self.push_points(&mut points, true);
            self.adjust_intersection_at_edges(&mut points);
        }

        if points.len() < 2 {
            return None;
        }

        Some(Intersection { points, looped })
    }

    fn find_first_point(&self) -> Option<IntersectionPoint> {
        match (self.same, self.guide_point) {
            (false, None) => self.find_common_point_stochastic_distinct(),
            (false, Some(guide)) => self.find_common_point_with_guide_distinct(guide),
            (true, None) => self.find_common_point_stochastic_same(),
            (true, Some(guide)) => self.find_common_point_with_guide_same(guide),
        }
    }

    fn find_common_point_with_guide_distinct(
        &self,
        guide: Point3<f64>,
    ) -> Option<IntersectionPoint> {
        let projection_0 = self.find_point_projection(self.surface_0, guide);
        let projection_1 = self.find_point_projection(self.surface_1, guide);

        self.find_common_surface_point(projection_0, projection_1)
    }

    fn find_common_point_with_guide_same(&self, guide: Point3<f64>) -> Option<IntersectionPoint> {
        let point_0 = self.find_point_projection(self.surface_0, guide);
        let surface_1_distribution = self.surface_1.parameter_distribution();

        let mut rng = self.rng.borrow_mut();

        for _ in 0..Self::STOCHASTIC_FIRST_POINT_TRIES {
            let point_1 = surface_1_distribution.sample(&mut *rng);

            let common_point = self.find_common_surface_point(point_0, point_1);

            check_stochastic_points!(self, common_point);
        }

        None
    }

    fn find_common_point_stochastic_distinct(&self) -> Option<IntersectionPoint> {
        let surface_0_distribution = self.surface_0.parameter_distribution();
        let mut rng = self.rng.borrow_mut();

        for _ in 0..Self::STOCHASTIC_FIRST_POINT_TRIES {
            let point_0 = surface_0_distribution.sample(&mut *rng);

            let surface_0_point = self.surface_0.value(&point_0);
            let point_1 = self.find_point_projection(self.surface_1, surface_0_point);

            let common_point = self.find_common_surface_point(point_0, point_1);

            if common_point.is_some() {
                return common_point;
            }
        }

        None
    }

    fn find_common_point_stochastic_same(&self) -> Option<IntersectionPoint> {
        let surface_0_distribution = self.surface_0.parameter_distribution();
        let surface_1_distribution = self.surface_1.parameter_distribution();

        let mut rng = self.rng.borrow_mut();

        for _ in 0..Self::STOCHASTIC_FIRST_POINT_TRIES {
            let point_0 = surface_0_distribution.sample(&mut *rng);
            let point_1 = surface_1_distribution.sample(&mut *rng);

            let common_point = self.find_common_surface_point(point_0, point_1);

            check_stochastic_points!(self, common_point);
        }

        None
    }

    fn find_point_projection(
        &self,
        surface: &dyn DifferentialParametricForm<2, 3>,
        point: Point3<f64>,
    ) -> Vector2<f64> {
        let surface_point_distance = SurfacePointL2DistanceSquared::new(surface, point);

        let mut gradient_descent = GradientDescent::new(&surface_point_distance);
        gradient_descent.step = self.numerical_step;
        gradient_descent.calculate()
    }

    fn find_common_surface_point(
        &self,
        start_0: Vector2<f64>,
        start_1: Vector2<f64>,
    ) -> Option<IntersectionPoint> {
        let surface_surface_distance =
            SurfaceSurfaceL2DistanceSquared::new(self.surface_0, self.surface_1);

        let mut gradient_descent = GradientDescent::new(&surface_surface_distance);
        gradient_descent.step = self.numerical_step;
        gradient_descent.starting_point = vector![start_0.x, start_0.y, start_1.x, start_1.y];

        let minimum = gradient_descent.calculate();
        let surface_0_minimum = vector![minimum.x, minimum.y];
        let surface_1_minimum = vector![minimum.z, minimum.w];
        let surface_0_val = self.surface_0.value(&surface_0_minimum);
        let surface_1_val = self.surface_1.value(&surface_1_minimum);

        if (surface_0_val - surface_1_val).norm() > self.numerical_step {
            return None;
        }

        let midpoint = point_avg(surface_0_val, surface_1_val);
        Some(IntersectionPoint {
            surface_0: surface_0_minimum,
            surface_1: surface_1_minimum,
            point: midpoint,
        })
    }

    fn next_intersection_point(
        &self,
        last_point: &IntersectionPoint,
        inverse_direction: bool,
    ) -> Option<IntersectionPoint> {
        let surface_0_arg = last_point.surface_0;
        let surface_1_arg = last_point.surface_1;

        let direction = self.common_tangent(&surface_0_arg, &surface_1_arg)
            * if inverse_direction { -1.0 } else { 1.0 };

        let step_function = IntersectionStepFunction::new(
            self.surface_0,
            self.surface_1,
            last_point.point,
            direction,
            self.intersection_step,
        );

        let mut newtons_algorithm = NewtonsAlgorithm::new(&step_function);
        newtons_algorithm.starting_point = vector![
            surface_0_arg.x,
            surface_0_arg.y,
            surface_1_arg.x,
            surface_1_arg.y
        ];
        newtons_algorithm.accuracy = self.numerical_step;

        newtons_algorithm.calculate().map(|solution| {
            let surface_0_arg = vector![solution.x, solution.y];
            let surface_1_arg = vector![solution.z, solution.w];
            let surface_0_point = self.surface_0.value(&surface_0_arg);
            let surface_1_point = self.surface_1.value(&surface_1_arg);

            let midpoint = point_avg(surface_0_point, surface_1_point);

            IntersectionPoint {
                surface_0: surface_0_arg,
                surface_1: surface_1_arg,
                point: midpoint,
            }
        })
    }

    fn common_tangent(
        &self,
        surface_0_arg: &Vector2<f64>,
        surface_1_arg: &Vector2<f64>,
    ) -> Vector3<f64> {
        let surface_0_normal = self.surface_0.normal(surface_0_arg);
        let surface_1_normal = self.surface_1.normal(surface_1_arg);

        Vector3::cross(&surface_0_normal, &surface_1_normal).normalize()
    }

    /// Returns true if the point sequence loops, expects `points` to contain the first
    /// intersection point
    fn push_points(&self, points: &mut Vec<IntersectionPoint>, inverse_direction: bool) -> bool {
        let loop_distances = self.parameter_space_backward_distances(&points[0]);

        if let Some(loop_distances) = (!inverse_direction).then_some(loop_distances).flatten() {
            self.push_points_with_loop_check(points, loop_distances)
        } else {
            self.push_points_without_loop_check(points, inverse_direction);
            false
        }
    }

    fn push_points_with_loop_check(
        &self,
        points: &mut Vec<IntersectionPoint>,
        loop_distances: (f64, f64),
    ) -> bool {
        while points.len() < Self::MAX_POINTS {
            let Some(next_intersection) =
                self.next_intersection_point(points.last().unwrap(), false)
            else {
                return false;
            };

            points.push(next_intersection);

            if points.len() >= 4
                && self.has_looped(&points[0], points.last().unwrap(), loop_distances)
            {
                return true;
            }
        }

        false
    }

    fn push_points_without_loop_check(
        &self,
        points: &mut Vec<IntersectionPoint>,
        inverse_direction: bool,
    ) {
        while points.len() < Self::MAX_POINTS {
            let Some(next_intersection) =
                self.next_intersection_point(points.last().unwrap(), inverse_direction)
            else {
                return;
            };

            points.push(next_intersection);
        }
    }

    /// Previous intersection points (when goint in the regular direction) in parameter space for both surfaces
    fn parameter_space_backward_distances(&self, point: &IntersectionPoint) -> Option<(f64, f64)> {
        let backward_point = self.next_intersection_point(point, true);
        backward_point.map(
            |IntersectionPoint {
                 surface_0,
                 surface_1,
                 ..
             }| {
                let distance_0 = self
                    .surface_0
                    .parameter_distance(&surface_0, &point.surface_0);

                let distance_1 = self
                    .surface_1
                    .parameter_distance(&surface_1, &point.surface_1);

                (distance_0, distance_1)
            },
        )
    }

    fn has_looped(
        &self,
        first_point: &IntersectionPoint,
        last_point: &IntersectionPoint,
        loop_distances: (f64, f64),
    ) -> bool {
        let distance_0 = self
            .surface_0
            .parameter_distance(&first_point.surface_0, &last_point.surface_0);

        let distance_1 = self
            .surface_1
            .parameter_distance(&first_point.surface_1, &last_point.surface_1);

        loop_distances.0 > distance_0 && loop_distances.1 > distance_1
    }

    fn adjust_intersection_at_edges(&self, points: &mut Vec<IntersectionPoint>) {
        let new_first = self.tightening_point(points[0]);
        let new_last = self.tightening_point(*points.last().unwrap());

        if let Some(new_first) = new_first {
            if Vector3::metric_distance(&new_first.point.coords, &points[0].point.coords)
                < self.intersection_step / 3.0
            {
                points[0] = new_first;
            } else {
                points.insert(0, new_first);
            }
        }

        if let Some(new_last) = new_last {
            if Vector3::metric_distance(&new_last.point.coords, &points[0].point.coords)
                < self.intersection_step / 3.0
            {
                let last_idx = points.len() - 1;
                points[last_idx] = new_last;
            } else {
                points.push(new_last);
            }
        }
    }

    fn tightening_point(&self, point: IntersectionPoint) -> Option<IntersectionPoint> {
        if !self.surface_0.wrapped(0)
            && !self.surface_0.wrapped(1)
            && self.surface_1.wrapped(0)
            && self.surface_1.wrapped(1)
        {}

        tighten_1_dim!(self, point, surface_0, 0, surface_1);
        tighten_1_dim!(self, point, surface_0, 1, surface_1);
        tighten_1_dim!(self, point, surface_1, 0, surface_0);
        tighten_1_dim!(self, point, surface_1, 1, surface_0);

        None
    }
}
