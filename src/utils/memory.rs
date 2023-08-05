pub fn bin_string_to_vec<T: Clone>(bin_string: &[u8]) -> Vec<T> {
    debug_assert!(bin_string.len() % std::mem::size_of::<T>() == 0);

    let slice = unsafe {
        std::slice::from_raw_parts(
            bin_string.as_ptr() as *const T,
            bin_string.len() / std::mem::size_of::<T>(),
        )
    };

    slice.to_vec()
}
