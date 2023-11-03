use crate::render::mesh::SurfaceVertex;

use super::{curvable::Curvable, gridable::Gridable};
use itertools::Itertools;
use nalgebra::{Point, Point3, SMatrix, SVector, Vector1, Vector2, Vector3};
use rand::{distributions::Uniform, prelude::Distribution, Rng};

pub trait ParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM>;
    fn value(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM>;
}

pub trait DifferentialParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM>;
    fn wrapped(&self, dim: usize) -> bool;
    fn value(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM>;
    fn jacobian(&self, vec: &SVector<f64, IN_DIM>) -> SMatrix<f64, OUT_DIM, IN_DIM>;
    fn hessian(
        &self,
        _vec: &SVector<f64, IN_DIM>,
        _var_0: usize,
        _var_1: usize,
    ) -> SVector<f64, OUT_DIM> {
        unimplemented!("Hessian not implemented for a differential parametric form");
    }

    fn parameter_distance(
        &self,
        first: &SVector<f64, IN_DIM>,
        second: &SVector<f64, IN_DIM>,
    ) -> f64 {
        let bounds_range = self.bounds().map(|coord| coord.1 - coord.0);

        (0..IN_DIM)
            .map(|dim| {
                if self.wrapped(dim) {
                    vec![-1.0, 0.0, 1.0]
                } else {
                    vec![0.0]
                }
            })
            .multi_cartesian_product()
            .map(|shifts| {
                let mut second_shifted = *second;
                for (dim, shift) in shifts.iter().enumerate() {
                    *second_shifted.get_mut(dim).unwrap() += shift * bounds_range[dim];
                }

                SVector::metric_distance(first, &second_shifted)
            })
            .min_by(f64::total_cmp)
            .unwrap()
    }

    fn parameter_distribution(&self) -> ParameterDistribution<IN_DIM> {
        ParameterDistribution {
            distribution: self.bounds().map(|b| Uniform::new_inclusive(b.0, b.1)),
        }
    }
}

pub struct ParameterDistribution<const IN_DIM: usize> {
    distribution: SVector<Uniform<f64>, IN_DIM>,
}

impl<const IN_DIM: usize> ParameterDistribution<IN_DIM> {
    pub fn sample(&self, rng: &mut impl Rng) -> SVector<f64, IN_DIM> {
        self.distribution.map(|d| d.sample(rng))
    }
}

pub trait WithNormals {
    fn normal(&self, vec: &Vector2<f64>) -> Vector3<f64>;
}

impl<T: ?Sized> WithNormals for T
where
    T: DifferentialParametricForm<2, 3>,
{
    fn normal(&self, vec: &Vector2<f64>) -> Vector3<f64> {
        let jacobian = self.jacobian(vec);
        jacobian
            .fixed_view::<3, 1>(0, 0)
            .cross(&jacobian.fixed_view::<3, 1>(0, 1))
    }
}

impl<const IN_DIM: usize, const OUT_DIM: usize, T> ParametricForm<IN_DIM, OUT_DIM> for T
where
    T: DifferentialParametricForm<IN_DIM, OUT_DIM>,
{
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM> {
        self.bounds()
    }

    fn value(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM> {
        self.value(vec)
    }
}

fn filtered_curve_thread<F: Fn(&Point3<f32>) -> bool + Send + Copy, P: ParametricForm<1, 3>>(
    samples: usize,
    th: usize,
    samples_per_thread: usize,
    form: &P,
    filter: F,
) -> (Vec<Point3<f32>>, Vec<u32>) {
    let mut points = Vec::with_capacity(samples);
    let mut indices = Vec::with_capacity(2 * samples);

    // Allow overlap between segments so that the curve is without breaks
    let lower = samples_per_thread * th;
    let upper = std::cmp::min(samples_per_thread * (th + 1), samples - 1);
    for i in lower..=upper {
        let range = form.bounds().x.1 - form.bounds().x.0;
        let t = i as f64 / (samples - 1) as f64 * range + form.bounds().x.0;

        let point = form.value(&Vector1::new(t));
        let point = Point3::new(point.x as f32, point.y as f32, point.z as f32);
        if filter(&point) {
            let idx = points.len();
            points.push(point);

            if idx != 0 {
                indices.push(idx as u32 - 1);
                indices.push(idx as u32);
            }
        }
    }

    (points, indices)
}

impl<T: ParametricForm<1, 3> + Sync> Curvable for T {
    fn filtered_curve<F: Fn(&Point3<f32>) -> bool + Send + Copy>(
        &self,
        samples: usize,
        filter: F,
    ) -> (Vec<Point3<f32>>, Vec<u32>) {
        if samples == 0 {
            return (Vec::new(), Vec::new());
        }

        const THREADS: usize = 16;
        let samples_per_thread = (samples - 1) / THREADS + 1;

        std::thread::scope(|scope| {
            let mut handles = Vec::new();

            for th in 0..THREADS {
                handles.push(scope.spawn({
                    move || filtered_curve_thread(samples, th, samples_per_thread, self, filter)
                }));
            }

            let mut points = Vec::new();
            let mut indices = Vec::new();
            for (batch_points, batch_indices) in
                handles.into_iter().map(|handle| handle.join().unwrap())
            {
                points.push(batch_points);
                indices.push(batch_indices);
            }

            // Make indices consistent between results from different threads
            let mut points_sum = 0;
            for i in 1..THREADS {
                points_sum += points[i - 1].len();
                for indice in &mut indices[i] {
                    *indice += points_sum as u32;
                }
            }

            (points.concat(), indices.concat())
        })
    }
}

impl<T: ParametricForm<2, 3>> Gridable for T {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<SurfaceVertex>, Vec<u32>) {
        let point_count = (points_x + 1) * (points_y + 1);
        let mut points = Vec::with_capacity(point_count as usize);
        let mut indices = Vec::with_capacity(2 * point_count as usize);

        for x_idx in 0..(points_x + 1) {
            for y_idx in 0..(points_y + 1) {
                let x_range = self.bounds().x.1 - self.bounds().x.0;
                let x = x_idx as f64 / points_x as f64 * x_range + self.bounds().x.0;

                let y_range = self.bounds().y.1 - self.bounds().y.0;
                let y = y_idx as f64 / points_y as f64 * y_range + self.bounds().y.0;

                let point = self.value(&Vector2::new(x, y));
                let point_idx = points.len() as u32;
                points.push(SurfaceVertex {
                    point: Point3::new(point.x as f32, point.y as f32, point.z as f32),
                    uv: Vector2::new(x as f32, y as f32),
                });

                indices.push(point_idx);
                indices.push((y_idx + 1) % (points_y + 1) + x_idx * (points_y + 1));
                indices.push(point_idx);
                indices.push((point_idx + points_y + 1) % point_count);
            }
        }

        (points, indices)
    }
}
