use super::geometry::parametric_form::DifferentialParametricForm;
use nalgebra::{SVector, Vector4, LU};

pub struct NewtonsAlgorithm<'f, const DIM: usize> {
    pub function: &'f dyn DifferentialParametricForm<DIM, DIM>,
    pub starting_point: SVector<f64, DIM>,
    pub max_iterations: usize,
    pub accuracy: f64,
}

impl<'f> NewtonsAlgorithm<'f, 4> {
    const DIM: usize = 4;
    pub fn new(function: &'f dyn DifferentialParametricForm<4, 4>) -> Self {
        Self {
            function,
            starting_point: SVector::zeros(),
            max_iterations: 100,
            accuracy: 0.0001,
        }
    }

    pub fn calculate(&self) -> Option<Vector4<f64>> {
        let mut current_arg = self.starting_point;
        let bounds = self.function.bounds();

        for _ in 0..self.max_iterations {
            let system = LU::new(self.function.jacobian(&current_arg));

            // The solution is (x_{n+1} - x_n)
            let Some(solution) = system.solve(&-self.function.value(&current_arg).coords)
            else {
                return None;
            };

            let mut new_arg = solution + current_arg;

            for dim in 0..Self::DIM {
                if self.function.wrapped(dim) {
                    new_arg[dim] = (new_arg[dim] - bounds[dim].0)
                        .rem_euclid(bounds[dim].1 - bounds[dim].0)
                        + bounds[dim].0;
                } else if new_arg[dim] < bounds[dim].0 || new_arg[dim] > bounds[dim].1 {
                    return None;
                }
            }

            current_arg = new_arg;

            if self.function.value(&current_arg).coords.norm_squared() < self.accuracy {
                return Some(current_arg);
            }
        }

        None
    }
}
