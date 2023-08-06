// use wasm_bindgen::JsValue;

pub fn slice_to_bin_string<T>(val: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            val.as_ptr() as *const u8,
            std::mem::size_of_val(val),
        )
    }
}

pub fn struct_to_bin_string<T>(val: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(val as *const T as *const u8, core::mem::size_of::<T>()) }
}

