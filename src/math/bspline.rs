use super::bernstein_polynomial::BernsteinPolynomial;

#[derive(Clone, Debug)]
pub struct CubicBSpline {
    bernsteins: Vec<BernsteinPolynomial<f64>>,
    deboor_points: Vec<f64>,
}

impl CubicBSpline {
    pub fn with_coefficients(deboor_points: Vec<f64>) -> Self {
        Self {
            bernsteins: Self::as_cubic_c0(&deboor_points),
            deboor_points,
        }
    }

    fn as_cubic_c0(deboor_points: &[f64]) -> Vec<BernsteinPolynomial<f64>> {
        let mut bernsteins = Vec::new();

        for i in 0..deboor_points.len() - 1 {
            bernsteins.push(BernsteinPolynomial::with_coefficients(vec![
                0.0,
                (2.0 * deboor_points[i] + deboor_points[i + 1]) / 3.0,
                (deboor_points[i] + 2.0 * deboor_points[i + 1]) / 3.0,
                0.0,
            ]));
        }

        for i in 1..deboor_points.len() - 2 {
            bernsteins[i].coeffs[0] = (bernsteins[i - 1].coeffs[2] + bernsteins[i].coeffs[1]) * 0.5;
            bernsteins[i].coeffs[3] = (bernsteins[i].coeffs[2] + bernsteins[i + 1].coeffs[1]) * 0.5;
        }

        bernsteins[1..deboor_points.len() - 2].to_vec()
    }

    pub fn value(&self, t: f64) -> f64 {
        let curve_idx = if t == 1.0 {
            self.bernsteins.len() - 1
        } else {
            (t * self.bernsteins.len() as f64).floor() as usize
        };

        let curve_t = self.bernsteins.len() as f64 * t - curve_idx as f64;
        self.bernsteins[curve_idx].value(curve_t)
    }

    pub fn modify_bernstein(&self, point_idx: usize, val: f64) -> Self {
        let mut new_deboor = self.deboor_points.clone();

        let (segment_idx, knot_idx) = if point_idx == 0 {
            (0, 0)
        } else {
            ((point_idx - 1) / 3, (point_idx - 1) % 3 + 1)
        };

        if point_idx % 3 == 0 {
            new_deboor[point_idx / 3 + 1] +=
                1.5 * (val - self.bernsteins[segment_idx].coeffs[knot_idx]);
        } else {
            new_deboor[segment_idx + 1] += val - self.bernsteins[segment_idx].coeffs[knot_idx];
            new_deboor[segment_idx + 2] += val - self.bernsteins[segment_idx].coeffs[knot_idx];
        }

        Self::with_coefficients(new_deboor)
    }

    pub fn bernstein_values(&self) -> Vec<f64> {
        let mut vals = Vec::new();
        for bernstein in &self.bernsteins {
            vals.push(bernstein.coeffs[0]);
            vals.push(bernstein.coeffs[1]);
            vals.push(bernstein.coeffs[2]);
        }

        vals.push(self.bernsteins.last().unwrap().coeffs[3]);
        vals
    }

    pub fn deboor_points(&self) -> Vec<f64> {
        self.deboor_points.clone()
    }
}
