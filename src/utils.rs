pub fn slice_as_raw<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        )
    }
}

/// For use with e.g. `Vec::retain`.
pub fn with_index<T, F>(mut f: F) -> impl FnMut(&T) -> bool
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = -1;
    move |item| {
        i += 1;
        f(i as usize, item)
    }
}

pub fn transpose_vector<T: Clone>(vec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut transpose = vec![Vec::<Option<T>>::new(); vec[0].len()];

    for i in 0..vec[0].len() {
        transpose[i].resize(vec.len(), None);
        for j in 0..vec.len() {
            transpose[i][j] = Some(vec[j][i].clone());
        }
    }

    transpose
        .into_iter()
        .map(|v| v.into_iter().map(|e| e.unwrap()).collect())
        .collect()
}
