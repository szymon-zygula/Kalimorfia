use super::{curvable::Curvable, gridable::Gridable};
use nalgebra::{Point, Point3, SVector, Vector1, Vector2};

pub trait ParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    const PARAMETER_BOUNDS: SVector<(f64, f64), IN_DIM>;

    fn parametric(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM>;
}

impl<T: ParametricForm<1, 3>> Curvable for T {
    fn curve(&self, samples: usize) -> Vec<Point3<f32>> {
        let mut points = Vec::with_capacity(samples);

        for i in 0..samples {
            let range = Self::PARAMETER_BOUNDS.x.1 - Self::PARAMETER_BOUNDS.x.0;
            let t = i as f64 / (samples - 1) as f64 * range + Self::PARAMETER_BOUNDS.x.0;

            let point = self.parametric(&Vector1::new(t));
            points.push(Point3::new(point.x as f32, point.y as f32, point.z as f32));
        }

        points
    }
}

impl<T: ParametricForm<2, 3>> Gridable for T {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<Point3<f32>>, Vec<u32>) {
        let point_count = points_x * points_y;
        let mut points = Vec::with_capacity(point_count as usize);
        let mut indices = Vec::with_capacity(2 * point_count as usize);

        for x_idx in 0..points_x {
            for y_idx in 0..points_y {
                let x_range = Self::PARAMETER_BOUNDS.x.1 - Self::PARAMETER_BOUNDS.x.0;
                let x = x_idx as f64 / points_x as f64 * x_range + Self::PARAMETER_BOUNDS.x.0;

                let y_range = Self::PARAMETER_BOUNDS.y.1 - Self::PARAMETER_BOUNDS.y.0;
                let y = y_idx as f64 / points_y as f64 * y_range + Self::PARAMETER_BOUNDS.y.0;

                let point = self.parametric(&Vector2::new(x, y));
                let point_idx = points.len() as u32;
                points.push(Point3::new(point.x as f32, point.y as f32, point.z as f32));

                indices.push(point_idx);
                indices.push((y_idx + 1) % points_y + x_idx * points_y);
                indices.push(point_idx);
                indices.push((point_idx + points_y) % point_count);
            }
        }

        (points, indices)
    }
}
