use crate::traits::*;
use crate::types::*;

// i32

impl TryFromJsValue for i32 {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        val.int32_value(scope)
            .ok_or(TryFromJsValueError::new("not a i32"))
    }
}

impl GetJsValue for i32 {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::Integer::new(scope, *self).into())
    }
}

// u32

impl TryFromJsValue for u32 {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        val.uint32_value(scope)
            .ok_or(TryFromJsValueError::new("not a u32"))
    }
}

impl GetJsValue for u32 {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::Integer::new_from_unsigned(scope, *self).into())
    }
}

// i64

impl TryFromJsValue for i64 {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        val.integer_value(scope)
            .ok_or(TryFromJsValueError::new("not a i64"))
    }
}

impl GetJsValue for i64 {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::BigInt::new_from_i64(scope, *self).into())
    }
}

// f32

impl TryFromJsValue for f32 {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        val.number_value(scope)
            .ok_or(TryFromJsValueError::new("not a f32"))
            .map(|v| v as f32)
    }
}

impl GetJsValue for f32 {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::Number::new(scope, *self as f64).into())
    }
}

// f64

impl TryFromJsValue for f64 {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        val.number_value(scope)
            .ok_or(TryFromJsValueError::new("not a f64"))
    }
}

impl GetJsValue for f64 {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::Number::new(scope, *self).into())
    }
}

// bool

impl TryFromJsValue for bool {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        Ok(val.boolean_value(scope))
    }
}

impl GetJsValue for bool {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::Boolean::new(scope, *self).into())
    }
}

// String

impl TryFromJsValue for String {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        let s = val
            .to_string(scope)
            .ok_or(TryFromJsValueError::new("not a string"))?;
        let s = s.to_rust_string_lossy(scope);
        Ok(s)
    }
}

impl GetJsValue for String {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        let s = v8::String::new(scope, self).unwrap();
        Ok(s.into())
    }
}

// ()

impl TryFromJsValue for () {
    fn try_from<'a>(
        _scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        if !val.is_null_or_undefined() {
            return Err(TryFromJsValueError::new("not null or undefined"));
        }
        Ok(())
    }
}

impl GetJsValue for () {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        Ok(v8::undefined(scope).into())
    }
}
