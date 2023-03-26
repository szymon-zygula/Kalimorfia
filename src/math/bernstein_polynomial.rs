use nalgebra::RealField;

#[derive(Copy, Clone, Debug)]
pub struct BernsteinPolynomial<T: RealField + Copy> {
    pub coeffs: Vec<T>,
}

impl<T: RealField + Copy> BernsteinPolynomial<T> {
    pub fn with_coefficients(coeffs: Vec<T>) -> Self {
        Self { coeffs }
    }

    pub fn degree(&self) -> usize {
        self.coeffs.len() - 1
    }

    pub fn value(&self, t: T) -> T {
        let t1 = T::one() - t;

        let mut values = self.coeffs.clone();
        let mut values_swap = vec![T::zero(); values.len()];

        // De Casteljau algorithm
        for i in (1..=self.degree()).rev() {
            for j in 0..i {
                values_swap[j] = t1 * values[j] + t * values[j + 1];
            }

            std::mem::swap(&mut values, &mut values_swap);
        }

        values[0]
    }
}
