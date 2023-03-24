use nalgebra::{Matrix3, Matrix4, RealField, Vector3};

pub struct AxisAngleDecomposition<T: RealField + Copy> {
    pub angle: T,
    pub axis: Vector3<T>,
}

impl<T: RealField + Copy> AxisAngleDecomposition<T> {
    pub fn decompose(matrix: &Matrix4<T>) -> Self {
        if matrix.norm() == T::zero() {
            return AxisAngleDecomposition {
                angle: T::zero(),
                axis: Vector3::new(T::one(), T::zero(), T::zero()),
            };
        }

        let angle_cosine =
            (matrix.fixed_view::<3, 3>(0, 0).trace() - T::one()) * T::from_f32(0.5).unwrap();

        if angle_cosine == T::one() {
            return Self {
                angle: T::zero(),
                axis: Vector3::new(T::one(), T::zero(), T::zero()),
            };
        }

        let angle_sine = (T::one() - angle_cosine * angle_cosine).sqrt();

        let axis = if matrix[(2, 1)] == matrix[(1, 2)]
            && matrix[(0, 2)] == matrix[(2, 0)]
            && matrix[(1, 0)] == matrix[(0, 1)]
        {
            // Eigenvector corresponding to the eigenvalue 1.0
            (matrix.fixed_view::<3, 3>(0, 0) - Matrix3::identity())
                .lu()
                .u()
                .solve_upper_triangular(&Vector3::zeros())
                .unwrap_or(Vector3::new(T::one(), T::zero(), T::zero()))
                .normalize()
        } else {
            Vector3::new(
                matrix[(2, 1)] - matrix[(1, 2)],
                matrix[(0, 2)] - matrix[(2, 0)],
                matrix[(1, 0)] - matrix[(0, 1)],
            ) / angle_sine
                * T::from_f32(0.5).unwrap()
        };

        let axis = if axis.norm().is_finite() {
            axis
        } else {
            Vector3::zeros()
        };

        Self {
            angle: angle_cosine.acos(),
            axis,
        }
    }
}
