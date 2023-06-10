use super::functions::DifferentiableScalarFunction;
use nalgebra::SVector;

pub struct GradientDescent<'f, const DIM: usize> {
    pub function: &'f dyn DifferentiableScalarFunction<DIM>,
    pub starting_point: SVector<f64, DIM>,
    pub step_size: f64,
    pub max_iterations: usize,
}

impl<'f, const DIM: usize> GradientDescent<'f, DIM> {
    pub fn new(function: &'f dyn DifferentiableScalarFunction<DIM>) -> Self {
        Self {
            function,
            starting_point: SVector::zeros(),
            step_size: 0.0001,
            max_iterations: 100,
        }
    }

    /// Finds the minimum of the function
    pub fn gradient_descent(&self) -> SVector<f64, DIM> {
        let mut current_arg = self.starting_point;
        let mut current_val = self.function.val(&current_arg);
        let bounds = self.function.bounds();

        for _ in 0..self.max_iterations {
            let mut new_arg = current_arg - self.step_size * self.function.grad(&current_arg);

            for dim in 0..DIM {
                new_arg[dim] = if self.function.wrapped(dim) {
                    (new_arg[dim] - bounds[dim].0).rem_euclid(bounds[dim].1 - bounds[dim].0)
                        + bounds[dim].0
                } else {
                    new_arg[dim].clamp(bounds[dim].0, bounds[dim].1)
                };
            }

            let new_val = self.function.val(&new_arg);

            if new_val > current_val {
                break;
            }

            current_arg = new_arg;
            current_val = new_val;
        }

        current_arg
    }
}
