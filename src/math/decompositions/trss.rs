use crate::math::affine::transforms::shear_xy_xz_yz;
use nalgebra::{Matrix4, RealField};

#[derive(Clone, Copy, Debug)]
pub struct TRSSDecomposition<T: RealField + Copy> {
    pub translation: Matrix4<T>,
    pub rotation: Matrix4<T>,
    pub shear: Matrix4<T>,
    pub scale: Matrix4<T>,
}

impl<T: RealField + Copy> TRSSDecomposition<T> {
    pub fn decompose(mut matrix: Matrix4<T>) -> TRSSDecomposition<T> {
        let mut translation = Matrix4::identity();
        translation[(0, 3)] = matrix[(0, 3)];
        translation[(1, 3)] = matrix[(1, 3)];
        translation[(2, 3)] = matrix[(2, 3)];

        let mut major = matrix.fixed_view_mut::<3, 3>(0, 0);

        let mut scale = Matrix4::identity();

        scale[(0, 0)] = major.column(0).norm();
        major[(0, 0)] /= scale[(0, 0)];
        major[(1, 0)] /= scale[(0, 0)];
        major[(2, 0)] /= scale[(0, 0)];

        let mut sxy = major.column(0).dot(&major.column(1));
        major[(0, 1)] = major[(0, 1)] - major[(0, 0)] * sxy;
        major[(1, 1)] = major[(1, 1)] - major[(1, 0)] * sxy;
        major[(2, 1)] = major[(2, 1)] - major[(2, 0)] * sxy;

        scale[(1, 1)] = major.column(1).norm();
        major[(0, 1)] /= scale[(1, 1)];
        major[(1, 1)] /= scale[(1, 1)];
        major[(2, 1)] /= scale[(1, 1)];
        sxy /= scale[(1, 1)];

        let mut sxz = major.column(0).dot(&major.column(2));
        major[(0, 2)] = major[(0, 2)] - major[(0, 0)] * sxz;
        major[(1, 2)] = major[(1, 2)] - major[(1, 0)] * sxz;
        major[(2, 2)] = major[(2, 2)] - major[(2, 0)] * sxz;

        let mut syz = major.column(1).dot(&major.column(2));
        major[(0, 2)] = major[(0, 2)] - major[(0, 1)] * syz;
        major[(1, 2)] = major[(1, 2)] - major[(1, 1)] * syz;
        major[(2, 2)] = major[(2, 2)] - major[(2, 1)] * syz;
        println!("col2: {}", major.column(2));

        scale[(2, 2)] = major.column(2).norm();
        major[(0, 2)] /= scale[(2, 2)];
        major[(1, 2)] /= scale[(2, 2)];
        major[(2, 2)] /= scale[(2, 2)];
        sxz /= scale[(2, 2)];
        syz /= scale[(2, 2)];

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
            shear: shear_xy_xz_yz(sxy, sxz, syz),
            scale,
        }
    }
}
