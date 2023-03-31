use nalgebra::Point3;

pub trait Curvable {
    fn curve(&self, samples: usize) -> (Vec<Point3<f32>>, Vec<u32>) {
        self.filtered_curve(samples, |_| true)
    }

    fn filtered_curve<F: Fn(&Point3<f32>) -> bool + Send + Copy>(
        &self,
        samples: usize,
        filter: F,
    ) -> (Vec<Point3<f32>>, Vec<u32>);
}
