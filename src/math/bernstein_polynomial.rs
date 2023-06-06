use itertools::Itertools;
use nalgebra::RealField;

#[derive(Clone, Debug)]
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

    pub fn divide_at(&self, t: T) -> (Self, Self) {
        let mut coeffs0 = vec![self.coeffs[0]];
        let mut coeffs1 = vec![self.coeffs[self.degree()]];

        let t1 = T::one() - t;

        let mut values = self.coeffs.clone();
        let mut values_swap = vec![T::zero(); values.len()];

        for i in (1..=self.degree()).rev() {
            for j in 0..i {
                values_swap[j] = t1 * values[j] + t * values[j + 1];
            }

            std::mem::swap(&mut values, &mut values_swap);

            coeffs0.push(values[0]);
            coeffs1.push(values[i - 1]);
        }

        coeffs1.reverse();

        (
            Self::with_coefficients(coeffs0),
            Self::with_coefficients(coeffs1),
        )
    }

    pub fn derivative(&self, t: T) -> T {
        if self.coeffs.len() == 0 {
            return T::zero();
        }

        let degree = T::from_f64(self.coeffs.len() as f64).unwrap();

        // This is inefficient to do on every call to `derivative`
        let derivative_coeffs: Vec<_> = self
            .coeffs
            .iter()
            .tuple_windows()
            .map(|(&a0, &a1)| degree * (-a0 + a1))
            .collect();

        let derivative = BernsteinPolynomial::with_coefficients(derivative_coeffs);
        derivative.value(t)
    }
}
