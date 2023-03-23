use nalgebra::{Matrix4, RealField, Vector3};

/// Decomposes a homogeneous linear transformation `A` into translation `T`, rotation `R`,
/// shear `H` and scale `S` components so that `A=T*R*H*S`.
#[derive(Clone, Copy, Debug)]
pub struct TRSSDecomposition<T: RealField + Copy> {
    pub translation: Vector3<T>,
    pub rotation: Matrix4<T>,
    pub shear: Vector3<T>,
    pub scale: Vector3<T>,
}

impl<T: RealField + Copy> TRSSDecomposition<T> {
    /// Performs the algorithm presented in Graphics Gems II VII.2 (Decomposing A Matrix into
    /// Simple Transformations)
    pub fn decompose(mut matrix: Matrix4<T>) -> TRSSDecomposition<T> {
        let translation = Vector3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]);

        let mut major = matrix.fixed_view_mut::<3, 3>(0, 0);

        let mut scale = Vector3::zeros();

        scale.x = major.column(0).norm();
        if scale.x != T::zero() {
            major[(0, 0)] /= scale.x;
            major[(1, 0)] /= scale.x;
            major[(2, 0)] /= scale.x;
        }

        let mut sxy = major.column(0).dot(&major.column(1));
        major[(0, 1)] = major[(0, 1)] - major[(0, 0)] * sxy;
        major[(1, 1)] = major[(1, 1)] - major[(1, 0)] * sxy;
        major[(2, 1)] = major[(2, 1)] - major[(2, 0)] * sxy;

        scale.y = major.column(1).norm();
        if scale.y != T::zero() {
            major[(0, 1)] /= scale.y;
            major[(1, 1)] /= scale.y;
            major[(2, 1)] /= scale.y;
            sxy /= scale.y;
        }

        let mut sxz = major.column(0).dot(&major.column(2));
        major[(0, 2)] = major[(0, 2)] - major[(0, 0)] * sxz;
        major[(1, 2)] = major[(1, 2)] - major[(1, 0)] * sxz;
        major[(2, 2)] = major[(2, 2)] - major[(2, 0)] * sxz;

        let mut syz = major.column(1).dot(&major.column(2));
        major[(0, 2)] = major[(0, 2)] - major[(0, 1)] * syz;
        major[(1, 2)] = major[(1, 2)] - major[(1, 1)] * syz;
        major[(2, 2)] = major[(2, 2)] - major[(2, 1)] * syz;

        scale.z = major.column(2).norm();
        if scale.z != T::zero() {
            major[(0, 2)] /= scale.z;
            major[(1, 2)] /= scale.z;
            major[(2, 2)] /= scale.z;
            sxz /= scale.z;
            syz /= scale.z;
        }

        if major.determinant() < T::zero() {
            major *= -T::one();
        }

        let mut rotation = Matrix4::identity();
        rotation[(0, 0)] = major[(0, 0)];
        rotation[(0, 1)] = major[(0, 1)];
        rotation[(0, 2)] = major[(0, 2)];
        rotation[(1, 0)] = major[(1, 0)];
        rotation[(1, 1)] = major[(1, 1)];
        rotation[(1, 2)] = major[(1, 2)];
        rotation[(2, 0)] = major[(2, 0)];
        rotation[(2, 1)] = major[(2, 1)];
        rotation[(2, 2)] = major[(2, 2)];

        TRSSDecomposition {
            translation,
            rotation,
            shear: Vector3::new(sxy, sxz, syz),
            scale,
        }
    }
}
