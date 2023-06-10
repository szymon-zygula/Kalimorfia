use super::{curvable::Curvable, gridable::Gridable};
use nalgebra::{Point, Point3, SMatrix, SVector, Vector1, Vector2};

pub trait ParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM>;
    fn parametric(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM>;
}

pub trait DifferentialParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM>;
    fn wrapped(&self, dim: usize) -> bool;
    fn parametric(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM>;
    fn jacobian(&self, vec: &SVector<f64, IN_DIM>) -> SMatrix<f64, OUT_DIM, IN_DIM>;
}

impl<const IN_DIM: usize, const OUT_DIM: usize, T: DifferentialParametricForm<IN_DIM, OUT_DIM>>
    ParametricForm<IN_DIM, OUT_DIM> for T
{
    fn bounds(&self) -> SVector<(f64, f64), IN_DIM> {
        self.bounds()
    }

    fn parametric(&self, vec: &SVector<f64, IN_DIM>) -> Point<f64, OUT_DIM> {
        self.parametric(vec)
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

        let point = form.parametric(&Vector1::new(t));
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
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<Point3<f32>>, Vec<u32>) {
        let point_count = points_x * points_y;
        let mut points = Vec::with_capacity(point_count as usize);
        let mut indices = Vec::with_capacity(2 * point_count as usize);

        for x_idx in 0..points_x {
            for y_idx in 0..points_y {
                let x_range = self.bounds().x.1 - self.bounds().x.0;
                let x = x_idx as f64 / points_x as f64 * x_range + self.bounds().x.0;

                let y_range = self.bounds().y.1 - self.bounds().y.0;
                let y = y_idx as f64 / points_y as f64 * y_range + self.bounds().y.0;

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
