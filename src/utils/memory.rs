pub trait TPod {}

impl TPod for u8 {}
impl TPod for u16 {}
impl TPod for u32 {}
impl TPod for u64 {}
impl TPod for u128 {}
impl TPod for usize {}
impl TPod for i8 {}
impl TPod for i16 {}
impl TPod for i32 {}
impl TPod for i64 {}
impl TPod for i128 {}
impl TPod for isize {}
impl TPod for f32 {}
impl TPod for f64 {}
impl<T: TPod, const C: usize, const R: usize> TPod for nalgebra_glm::TMat<T, C, R> {}

pub fn bin_string_to_vec<T: Clone + TPod>(bin_string: &[u8]) -> Vec<T> {
    debug_assert!(bin_string.len() % std::mem::size_of::<T>() == 0);

    let slice = unsafe {
        std::slice::from_raw_parts(
            bin_string.as_ptr() as *const T,
            bin_string.len() / std::mem::size_of::<T>(),
        )
    };

    slice.to_vec()
}
