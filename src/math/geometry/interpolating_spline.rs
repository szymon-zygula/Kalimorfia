use crate::math::decompositions::tridiagonal;
use itertools::Itertools;
use nalgebra::{Point3, RealField, Vector3};

type BernsteinTuple<T> = (Point3<T>, Point3<T>, Point3<T>, Point3<T>);

pub fn interpolating_spline_c2<T: RealField + Copy>(
    points: &[Point3<T>],
) -> Vec<BernsteinTuple<T>> {
    // Get rid of consecutive equal points so that we don't get any 0-length chords
    let points: Vec<_> = points.iter().copied().dedup().collect();

    let n = points.len() - 1;
    assert!(n >= 2);

    let chord_lengths: Vec<T> = points
        .iter()
        .tuple_windows()
        .map(|(p1, p2)| (p2 - p1).norm())
        .collect();

    let lower_diagonal: Vec<T> = chord_lengths
        .iter()
        .copied()
        .tuple_windows()
        .map(|(d1, d2)| d1 / (d1 + d2))
        .skip(1)
        .collect();

    let upper_diagonal: Vec<T> = chord_lengths
        .iter()
        .copied()
        .tuple_windows()
        .map(|(d1, d2)| d2 / (d1 + d2))
        .take(n - 2)
        .collect();

    let free_term: Vec<_> = points
        .iter()
        .copied()
        .tuple_windows()
        .zip(chord_lengths.iter().copied().tuple_windows())
        .map(|((p1, p2, p3), (d1, d2))| {
            let lhs = (p3 - p2) / d2;
            let rhs = (p2 - p1) / d1;
            let divisor = d1 + d2;
            ((lhs - rhs) / divisor) * T::from_f64(3.0).unwrap()
        })
        .collect();

    let c = tridiagonal::equation_system(
        [T::from_f64(2.0).unwrap()].repeat(n - 1),
        &lower_diagonal,
        &upper_diagonal,
        free_term,
    );

    let c: Vec<_> = [&[Vector3::zeros()], c.as_slice(), &[Vector3::zeros()]].concat();

    let d: Vec<_> = c
        .iter()
        .tuple_windows()
        .zip(chord_lengths.iter().copied())
        .map(|((c1, c2), chord)| (c2 - c1) / chord / T::from_f64(3.0).unwrap())
        .collect();

    let a: Vec<_> = points.iter().map(|p| p.coords).collect();

    let b: Vec<_> = itertools::multizip((
        a.iter().tuple_windows(),
        c.iter().copied(),
        d.iter().copied(),
        chord_lengths.iter().copied(),
    ))
    .map(|((a1, a2), c, d, chord)| (a2 - a1) / chord - c * chord - d * chord * chord)
    .collect();

    itertools::multizip((a, b, c, d, chord_lengths))
        .map(|(a, b, c, d, chord)| (a, b * chord, c * chord * chord, d * chord * chord * chord))
        .map(|(a, b, c, d)| {
            (
                Point3::from(a),
                Point3::from(a + b / T::from_f64(3.0).unwrap()),
                Point3::from(
                    a + b * T::from_f64(2.0 / 3.0).unwrap() + c / T::from_f64(3.0).unwrap(),
                ),
                Point3::from(a + b + c + d),
            )
        })
        .collect()
}
