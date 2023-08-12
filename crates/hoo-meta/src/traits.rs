use crate::types::*;

// Js 侧的 Object: 按照有无特殊标记区分
// 有特殊标记可以传递给 RcObject 和 普通 struct
// 没有特殊标记只能传递给 普通 struct

// 引擎返回 RcObject iff 有特殊标记
// 引擎尽量使用 RcObject

// 基础类型无法套 RcObject

pub trait TryFromJsValue {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError>
    where
        Self: Sized;
}

pub trait GetJsValue {
    // 就叫做 int：JsException 应当仅在 self 表示错误时抛出
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException>;
}

pub trait InitializeProperties {
    fn initialize_properties<'a>(
        scope: &mut v8::HandleScope<'a>,
    ) -> v8::Local<'a, v8::Object>;
}

// 填充 JsObject
// 接口可能会改，来彰显一下构造函数
pub trait FillJsObject {
    fn fill_js_object<'a: 'b, 'b>(
        &self,
        object: v8::Local<'b, v8::Object>,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<(), JsException>;
}
