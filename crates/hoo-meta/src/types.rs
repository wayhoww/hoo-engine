#[derive(Debug)]
pub struct TryFromJsValueError(String);

impl TryFromJsValueError {
    pub fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }

    pub fn error_message(&self) -> String {
        self.0.clone()
    }
}

// 将被转换为 js 异常抛出
#[derive(Debug)]
pub struct JsException(String);

impl JsException {
    pub fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }

    pub fn error_message(&self) -> String {
        self.0.clone()
    }
}
pub struct HooMetaContext<'a> {
    scope: v8::HandleScope<'a, ()>,
}

impl<'a> HooMetaContext<'a> {
    pub fn new(scope: v8::HandleScope<'a, ()>) -> Self {
        Self {
            scope,
        }
    }

    pub fn scope_mut(&mut self) -> &mut v8::HandleScope<'a, ()> {
        &mut self.scope
    }
}


