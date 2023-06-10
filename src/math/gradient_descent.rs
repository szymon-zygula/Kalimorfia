use super::functions::DifferentiableScalarFunction;
use nalgebra::SVector;

pub struct GradientDescent<'f, const DIM: usize> {
    function: &'f dyn DifferentiableScalarFunction<DIM>,
    pub starting_point: SVector<f64, DIM>,
    pub step: f64,
    pub max_iterations: usize,
    pub stop_epsilon: f64,
}

impl<'f, const DIM: usize> GradientDescent<'f, DIM> {
    pub fn new(function: &'f dyn DifferentiableScalarFunction<DIM>) -> Self {
        let bounds = function.bounds();
        let lower_bounds = bounds.map(|c| c.0);
        let upper_bounds = bounds.map(|c| c.1);

        Self {
            starting_point: (lower_bounds + upper_bounds) / 2.0,
            function,
            step: 0.0001,
            stop_epsilon: 1e-10,
            max_iterations: 10000,
        }
    }

    /// Finds the minimum of the function
    pub fn calculate(&self) -> SVector<f64, DIM> {
        let mut arg = self.starting_point;
        let mut val = self.function.val(&arg);
        let mut step = self.step;
        let bounds = self.function.bounds();

        for _ in 0..self.max_iterations {
            let mut new_arg = arg - step * self.function.grad(&arg);

            for dim in 0..DIM {
                new_arg[dim] = if self.function.wrapped(dim) {
                    (new_arg[dim] - bounds[dim].0).rem_euclid(bounds[dim].1 - bounds[dim].0)
                        + bounds[dim].0
                } else {
                    new_arg[dim].clamp(bounds[dim].0, bounds[dim].1)
                };
            }

            let new_val = self.function.val(&new_arg);

            if (new_val - val).abs() < self.stop_epsilon {
                return arg;
            }

            if new_val > val {
                step /= 2.0;
                continue;
            }

            arg = new_arg;
            val = new_val;
        }

        arg
    }
}
