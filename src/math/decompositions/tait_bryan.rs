use nalgebra::{Matrix4, RealField};

pub struct TaitBryanDecomposition<T: RealField + Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: RealField + Copy> TaitBryanDecomposition<T> {
    pub fn decompose(matrix: &Matrix4<T>) -> Self {
        Self {
            x: matrix[(2, 1)].atan2(matrix[(2, 2)]),
            y: (-matrix[(2, 0)])
                .atan2((matrix[(2, 1)] * matrix[(2, 1)] + matrix[(2, 2)] * matrix[(2, 2)]).sqrt()),
            z: matrix[(1, 0)].atan2(matrix[(0, 0)]),
        }
    }
}
