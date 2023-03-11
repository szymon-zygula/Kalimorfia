use nalgebra::SVector;

pub trait ParametricForm<const IN_DIM: usize, const OUT_DIM: usize> {
    fn parametric(&self, vec: &SVector<f64, IN_DIM>) -> SVector<f64, OUT_DIM>;
}
