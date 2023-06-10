use super::parametric_form::DifferentialParametricForm;
use nalgebra::{Point2, Point3};

pub struct IntersectionPoint {
    surface_0: Point2<f64>,
    surface_1: Point2<f64>,
    point: Point3<f64>,
}

pub struct Intersection {
    pub wrapped: bool,
    pub points: Vec<IntersectionPoint>,
}

impl Intersection {
    pub fn new(
        surface_0: &dyn DifferentialParametricForm<2, 3>,
        surface_1: &dyn DifferentialParametricForm<2, 3>,
    ) -> Self {
        Self {
            wrapped: todo!(),
            points: todo!(),
        }
    }
}
