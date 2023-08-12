use crate::traits::*;
use crate::types::*;

impl<OT: GetJsValue, ET: ToString> GetJsValue for Result<OT, ET> {
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        match self {
            Ok(i) => GetJsValue::get_js_value(i, scope),
            Err(e) => Err(JsException::new(&e.to_string())),
        }
    }
}
