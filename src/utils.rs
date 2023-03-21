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
