use wasm_bindgen::JsValue;

pub fn slice_to_bin_string<T>(val: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            val.as_ptr() as *const u8,
            core::mem::size_of::<T>() * val.len(),
        )
    }
}

pub fn struct_to_bin_string<T>(val: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(val as *const T as *const u8, core::mem::size_of::<T>()) }
}

pub fn jsarray<T: Into<JsValue> + ToOwned<Owned = T>>(x: &[T]) -> JsValue {
    JsValue::from(
        x.iter()
            .map(|x| Into::<JsValue>::into(x.to_owned()))
            .collect::<js_sys::Array>(),
    )
}
