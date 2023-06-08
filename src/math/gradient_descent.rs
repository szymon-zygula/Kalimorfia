use nalgebra::SVector;

pub trait DifferentiableScalarFunction<const DIM: usize> {
    fn val(&self, x: &SVector<f64, DIM>) -> f64;
    fn grad(&self, x: &SVector<f64, DIM>) -> SVector<f64, DIM>;
}

pub struct GradientDescent<'f, const DIM: usize> {
    pub function: &'f dyn DifferentiableScalarFunction<DIM>,
    pub starting_point: SVector<f64, DIM>,
    pub step_size: f64,
    pub max_iterations: usize,
}

impl<'f, const DIM: usize> GradientDescent<'f, DIM> {
    fn new(function: &'f dyn DifferentiableScalarFunction<DIM>) -> Self {
        Self {
            function,
            starting_point: SVector::zeros(),
            step_size: 0.001,
            max_iterations: 100,
        }
    }

    /// Finds the minimum of the function
    pub fn gradient_descent(&self) -> SVector<f64, DIM> {
        let mut current_arg = self.starting_point;
        let mut current_val = self.function.val(&current_arg);

        for _ in 0..self.max_iterations {
            let new_arg = current_arg - self.step_size * self.function.grad(&current_arg);
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
