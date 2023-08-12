#![allow(warnings, unused)]
#![allow(warnings, unused_imports)]

use hoo_object::RcObject;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    os::raw::c_void,
    rc::Rc,
};
use uuid::{uuid, Uuid};

use hoo_meta_macros::{get_js_function, js_function, js_impl, js_struct};
// use hoo_object::{ObjectId, RcAny, RcObject, RcTrait};

// use hoo_meta::*;

#[js_struct]
#[derive(Debug)]
struct RsPoint {
    pub x: i32,
    y: i32,
}

#[js_impl]
impl RsPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn length(&self, offset: f64) -> f64 {
        ((self.x * self.x + self.y * self.y) as f64).sqrt() + offset
    }
}

impl Drop for RsPoint {
    fn drop(&mut self) {
        println!("RsPoint::drop");
    }
}

impl RsPoint {
    // 适用于 &self 和 &mut self
    pub fn hoo_meta_method_callback_length<'a, 's>(
        this: &hoo_object::RcObject<RsPoint>,
        scope: &'a mut v8::HandleScope<'s>,
        args: v8::FunctionCallbackArguments<'s>,
    ) -> Result<f64, hoo_meta::TryFromJsValueError> {
        let this = this.borrow();

        let arg0 = hoo_meta::TryFromJsValue::try_from(scope, &args.get(0))?;

        let result = this.length(arg0);
        return Ok(result);
    }
}

#[js_struct]
struct RsPointPair {
    pub a: RcObject<RsPoint>,
    pub b: RcObject<RsPoint>,
}

#[js_impl]
impl RsPointPair {
    pub fn new(a: RcObject<RsPoint>, b: RcObject<RsPoint>) -> Self {
        Self { a, b }
    }
}

impl Drop for RsPointPair {
    fn drop(&mut self) {
        println!("RsPointPair::drop");
    }
}

// impl hoo_meta::InitializeProperties for hoo_object::RcObject<RsPoint> {
//     fn __hoo_meta_initialize_properties<'a>(
//         &self,
//         scope: &mut v8::HandleScope<'a>,
//     ) -> v8::Local<'a, v8::Object> {
//         todo!();
//     }
// }

// fn rs_make_object_callback(
//     scope: &mut v8::HandleScope,
//     _: v8::FunctionCallbackArguments,
//     mut _retval: v8::ReturnValue,
// ) {
//     let object = v8::Object::new(scope);
//     let key = v8::String::new(scope, "key").unwrap();
//     let value = v8::String::new(scope, "value").unwrap();
//     object.set(scope, key.into(), value.into());

//     let rc = std::rc::Rc::new(std::cell::RefCell::new(None));
//     let weak = v8::Weak::with_guaranteed_finalizer(
//         scope,
//         object,
//         Box::new({
//             let rc = rc.clone();
//             move || {
//                 // 有一个不知道什么语言特性：这边变量名命名为 _ 的话，rc 依然会被销毁
//                 #[allow(unused_variables)]
//                 let moved_rc = rc; // 让 rc 存续
//                 println!("finalizer called!");
//             }
//         }),
//     );
//     rc.replace(Some(weak));

//     _retval.set(object.into());
// }

#[js_function]
fn rs_add(a: i32, b: i32) -> Result<i32, String> {
    if a > 0 && b > 0 {
        return Ok(a + b);
    } else {
        return Err("a or b is not positive".to_string());
    }
}

#[js_function]
fn rs_length(pt: RsPoint) -> i32 {
    // let pt = pt.borrow();
    pt.x + pt.y
}

#[js_function]
fn rs_mut_add_one(pt: hoo_object::RcObject<RsPoint>) {
    let mut pt = pt.borrow_mut();
    pt.x += 1;
    pt.y += 1;
}

// 本来就要一个一个 add，可以考虑重构一下，不依赖全局量
#[js_function]
fn rs_add_one(pt: RsPoint) -> hoo_object::RcObject<RsPoint> {
    hoo_object::RcObject::new(RsPoint {
        x: pt.x + 1,
        y: pt.y + 1,
    })
}

fn main() {
    // Initialize V8.
    v8::V8::set_flags_from_string("--expose-gc");
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    {
        // Create a new Isolate and make it the current one.
        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());

        // Create a stack-allocated handle scope.
        let scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = hoo_meta::HooMetaContext::new(scope);

        // Create a new context.
        let context_template = v8::ObjectTemplate::new(hoo_meta_context.scope_mut());

        // func_template.

        // context_template.set(
        //     v8::String::new(scope, "rs_make_object")
        //         .unwrap()
        //         .into(),
        //     v8::FunctionTemplate::new(scope, rs_make_object_callback).into(),
        // );

        context_template.set(
            v8::String::new(hoo_meta_context.scope_mut(), "rs_add")
                .unwrap()
                .into(),
            v8::FunctionTemplate::new(hoo_meta_context.scope_mut(), get_js_function!(rs_add))
                .into(),
        );
        context_template.set(
            v8::String::new(hoo_meta_context.scope_mut(), "rs_length")
                .unwrap()
                .into(),
            v8::FunctionTemplate::new(hoo_meta_context.scope_mut(), get_js_function!(rs_length))
                .into(),
        );
        context_template.set(
            v8::String::new(hoo_meta_context.scope_mut(), "rs_mut_add_one")
                .unwrap()
                .into(),
            v8::FunctionTemplate::new(
                hoo_meta_context.scope_mut(),
                get_js_function!(rs_mut_add_one),
            )
            .into(),
        );
        context_template.set(
            v8::String::new(hoo_meta_context.scope_mut(), "rs_add_one")
                .unwrap()
                .into(),
            v8::FunctionTemplate::new(hoo_meta_context.scope_mut(), get_js_function!(rs_add_one))
                .into(),
        );
        RsPoint::__hoo_meta_register_struct(&mut hoo_meta_context, context_template); // (&mut hoo_meta_context, context_template);
        RsPointPair::__hoo_meta_register_struct(&mut hoo_meta_context, context_template); // (&mut hoo_meta_context, context_template);

        let context =
            v8::Context::new_from_template(hoo_meta_context.scope_mut(), context_template);

        let scope = &mut v8::ContextScope::new(hoo_meta_context.scope_mut(), context);

        // 不知道这个 scope 是不是有命令行历史之类的东西，如果让 y 成为一句话的返回值，那么 finalizer 要到最后才会执行。

        {
            let code = v8::String::new(
                scope,
                r#"
                new RsPoint(1, 1);
                new RsPoint(1, 1);
                new RsPoint(1, 1);
                new RsPoint(1, 1);
                // let point_pair = new RsPointPair(point1, point1);
                1
            "#,
            )
            .unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            let result = script.run(scope).unwrap();
            let result = result.to_string(scope).unwrap();
            println!("{}", result.to_rust_string_lossy(scope));
        }

        scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
        {
            println!("=======================================");
            let code = v8::String::new(
                scope,
                r#"
                1
                "#,
            )
            .unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            let result = script.run(scope).unwrap();
            let result = result.to_string(scope).unwrap();
            println!("{}", result.to_rust_string_lossy(scope));
        }        
        {
            println!("=======================================");
            let code = v8::String::new(
                scope,
                r#"
                1
                "#,
            )
            .unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            let result = script.run(scope).unwrap();
            let result = result.to_string(scope).unwrap();
            println!("{}", result.to_rust_string_lossy(scope));
        }
    }

    println!("=======================================");

    unsafe {
        v8::V8::dispose();
    }
    v8::V8::dispose_platform();
}
