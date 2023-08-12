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

pub struct HooMetaContext<'s, 'a> {
    context_scope: v8::ContextScope<'a, v8::HandleScope<'s>>,
    _context: v8::Local<'s, v8::Context>,
}

pub struct HooMetaModuleBuilder<'s, 'a, 'x> {
    context_builder: &'x mut HooMetaContextBuilder<'s, 'a>,
    object_template: v8::Local<'s, v8::ObjectTemplate>,
}

pub struct HooMetaContextBuilder<'s, 'a>
where
    's: 'a,
{
    global_scope: &'a mut v8::HandleScope<'s, ()>,
    context_template: v8::Local<'s, v8::ObjectTemplate>,
}

pub trait ModuleLikeBuilder<'s, 'a> {
    fn get_global_scope<'x>(&'x mut self) -> &'x mut v8::HandleScope<'s, ()>;
    fn get_template(&mut self) -> v8::Local<'s, v8::ObjectTemplate>;

    fn add_function(&mut self, name: &str, callback: impl v8::MapFnTo<v8::FunctionCallback>) {
        let key = v8::String::new(self.get_global_scope(), name).unwrap();
        let value = v8::FunctionTemplate::new(self.get_global_scope(), callback);
        self.get_template().set(key.into(), value.into());
    }

    fn add_class<F: FnOnce(&mut Self)>(&mut self, generator: F) {
        generator(self);
    }
}

impl<'s, 'a> ModuleLikeBuilder<'s, 'a> for HooMetaContextBuilder<'s, 'a> {
    fn get_global_scope<'x>(&'x mut self) -> &'x mut v8::HandleScope<'s, ()> {
        self.global_scope
    }

    fn get_template(&mut self) -> v8::Local<'s, v8::ObjectTemplate> {
        self.context_template
    }
}

impl<'s, 'a, 'x> ModuleLikeBuilder<'s, 'a> for HooMetaModuleBuilder<'s, 'a, 'x> {
    fn get_global_scope<'y>(&'y mut self) -> &'y mut v8::HandleScope<'s, ()> {
        self.context_builder.global_scope
    }

    fn get_template(&mut self) -> v8::Local<'s, v8::ObjectTemplate> {
        self.object_template
    }
}

impl<'s, 'a> HooMetaContextBuilder<'s, 'a>
where
    's: 'a,
{
    pub fn build_module<'x, F: FnOnce(&mut HooMetaModuleBuilder)>(
        &'x mut self,
        name: &str,
        build: F,
    ) {
        let object_template: v8::Local<'s, v8::ObjectTemplate> =
            v8::ObjectTemplate::new(self.global_scope);
        let key = v8::String::new(self.global_scope, name).unwrap();

        let mut module_builder: HooMetaModuleBuilder<'s, 'a, 'x> = HooMetaModuleBuilder {
            context_builder: self,
            object_template: object_template.clone(),
        };
        build(&mut module_builder);

        module_builder
            .context_builder
            .context_template
            .set(key.into(), object_template.into());
    }
}

impl<'s, 'a, 'x> HooMetaModuleBuilder<'s, 'a, 'x> {
    pub fn build_module<'y, F: FnOnce(&mut HooMetaModuleBuilder)>(
        &'y mut self,
        name: &str,
        build: F,
    ) {
        let object_template: v8::Local<'s, v8::ObjectTemplate> =
            v8::ObjectTemplate::new(self.context_builder.global_scope);
        let key = v8::String::new(self.context_builder.global_scope, name).unwrap();

        let mut module_builder: HooMetaModuleBuilder<'s, 'a, 'y> = HooMetaModuleBuilder {
            context_builder: self.context_builder,
            object_template: object_template.clone(),
        };
        build(&mut module_builder);

        self.object_template.set(key.into(), object_template.into());
    }
}

#[macro_export]
macro_rules! module_add_function {
    ($module_builder: tt, $function_name: tt) => {
        $module_builder.add_function(
            hoo_meta_macros::get_js_function_name_string!($function_name),
            hoo_meta_macros::get_js_function!($function_name),
        );
    };
}

#[macro_export]
macro_rules! module_add_class {
    ($module_builder: tt, $class_name: tt) => {
        $module_builder.add_class($class_name::__hoo_meta_register_struct);
    };
}

pub fn build_context<'a, 's: 'a, F: FnOnce(&mut HooMetaContextBuilder)>(
    global_scope: &'a mut v8::HandleScope<'s, ()>,
    build: F,
) -> HooMetaContext<'s, 'a> {
    // global_scope<> -> context_template<>
    let context_template: v8::Local<'s, v8::ObjectTemplate> = v8::ObjectTemplate::new(global_scope);

    let mut builder: HooMetaContextBuilder<'s, 'a> = HooMetaContextBuilder {
        global_scope,
        context_template,
    };

    build(&mut builder);

    // global_scope<> -> context<>
    let context: v8::Local<'s, v8::Context> =
        v8::Context::new_from_template(&mut builder.global_scope, context_template.clone());

    // global_scope<> and its self -> context_scope<>
    let context_scope: v8::ContextScope<'a, v8::HandleScope<'s>> =
        v8::ContextScope::new(builder.global_scope, context);

    HooMetaContext {
        context_scope,
        _context: context,
    }
}

impl<'s, 'a> HooMetaContext<'s, 'a> {
    pub fn scope_mut(&mut self) -> &mut v8::ContextScope<'a, v8::HandleScope<'s>> {
        &mut self.context_scope
    }

    pub fn evaluate_script(&mut self, source: &str) -> Option<v8::Local<'s, v8::Value>> {
        let scope = self.scope_mut();
        let code = v8::String::new(scope, source).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();
        script.run(scope)
    }

    pub fn evaluate_script_get_string(&mut self, source: &str) -> String {
        let result = self.evaluate_script(source).unwrap();
        let scope = self.scope_mut();
        let result = result.to_string(scope).unwrap().to_rust_string_lossy(scope);
        result
    }
}
