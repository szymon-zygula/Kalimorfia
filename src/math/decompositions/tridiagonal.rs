use nalgebra::{ClosedDiv, ClosedMul, ClosedSub};

pub fn equation_system<T, U>(
    mut diagonal: Vec<T>,
    lower_diagonal: &[T],
    upper_diagonal: &[T],
    mut free_term: Vec<U>,
) -> Vec<U>
where
    T: ClosedMul<T> + ClosedDiv<T> + ClosedSub<T> + Copy,
    U: ClosedMul<T> + ClosedDiv<T> + ClosedSub<U> + Copy,
{
    let n = diagonal.len();
    assert_eq!(n - 1, lower_diagonal.len());
    assert_eq!(n - 1, upper_diagonal.len());
    assert_eq!(n, free_term.len());

    // Not setting lower_diagonal to 0 because it is not used later anyway
    for i in 1..n {
        let multiplier = lower_diagonal[i - 1] / diagonal[i - 1];
        diagonal[i] -= multiplier * upper_diagonal[i - 1];

        let term = free_term[i - 1];
        free_term[i] -= term * multiplier;
    }

    // Same here for upper_diagonal and diagonal
    for i in (1..n).rev() {
        let multiplier = upper_diagonal[i - 1] / diagonal[i];
        let term = free_term[i];

        free_term[i - 1] -= term * multiplier;
        free_term[i] /= diagonal[i];
    }

    free_term[0] /= diagonal[0];

    free_term
}
