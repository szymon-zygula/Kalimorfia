use nalgebra::RealField;

#[derive(Copy, Clone, Debug)]
pub struct BernsteinPolynomial<T: RealField + Copy, const N: usize> {
    pub coeffs: [T; N],
}

impl<T: RealField + Copy, const N: usize> BernsteinPolynomial<T, N> {
    pub fn with_coefficients(coeffs: [T; N]) -> Self {
        Self { coeffs }
    }

    pub fn degree() -> usize {
        N - 1
    }

    pub fn value(&self, t: T) -> T {
        let t1 = T::one() - t;

        // Boxes so that `std::mem::swap` doesn't copy the arrays element by element
        let mut values = Box::new(self.coeffs);
        let mut values_swap = Box::new([T::zero(); N]);

        // De Casteljau algorithm
        for i in (1..N).rev() {
            for j in 0..i {
                values_swap[j] = t1 * values[j] + t * values[j + 1];
            }

            std::mem::swap(&mut values, &mut values_swap);
        }

        values[0]
    }
}
