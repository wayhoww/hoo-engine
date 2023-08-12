#[cfg(test)]
mod tests {

    // extern crate hoo_meta_macros;

    use crate as hoo_meta;
    use crate::*;
    use hoo_meta_macros::*;
    use hoo_object::RcObject;

    fn initialize() {
        crate::initialize("--expose-gc");
    }

    #[test]
    fn primitive_type() {
        initialize();

        #[js_function]
        fn succ(val: i32) -> i32 {
            return val + 1;
        }

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let mut global_scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = build_context(&mut global_scope, |context_builder| {
            module_add_function!(context_builder, succ);
        });

        assert_eq!(hoo_meta_context.evaluate_script_get_string("succ(1)"), "2");
    }

    #[test]
    fn value_type() {
        initialize();

        #[derive(JsStructNoConstructor)]
        struct Pair {
            pub x: i32,
            pub y: i32,
        }

        #[js_function]
        fn swap(val: Pair) -> Pair {
            return Pair { x: val.y, y: val.x };
        }

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let mut global_scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = build_context(&mut global_scope, |context_builder| {
            module_add_function!(context_builder, swap);
        });

        assert_eq!(
            hoo_meta_context.evaluate_script_get_string("swap({x: 1, y: 2}).x"),
            "2"
        );
    }

    #[test]
    fn reference_type() {
        initialize();

        #[derive(JsStruct)]
        struct Pair {
            pub x: i32,
            pub y: i32,
        }

        #[js_impl]
        impl Pair {
            pub fn new(x: i32, y: i32) -> Self {
                Self { x, y }
            }
        }

        #[js_function]
        fn new_pair(x: i32, y: i32) -> RcObject<Pair> {
            return RcObject::new(Pair::new(x, y));
        }

        #[js_function]
        fn swap(val: RcObject<Pair>) {
            let mut val = val.borrow_mut();
            let temp = val.x;
            val.x = val.y;
            val.y = temp;
        }

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let mut global_scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = build_context(&mut global_scope, |context_builder| {
            module_add_function!(context_builder, swap);
            module_add_function!(context_builder, new_pair);
            module_add_class!(context_builder, Pair);
        });

        assert!(hoo_meta_context
            .evaluate_script("swap({x: 1, y: 2})")
            .is_none());
        assert_eq!(
            hoo_meta_context.evaluate_script_get_string("let p = new Pair(1, 2); swap(p); p.x"),
            "2"
        );
        assert_eq!(
            hoo_meta_context.evaluate_script_get_string("let q = new_pair(1, 2); swap(q); q.x"),
            "2"
        );
    }

    #[test]
    fn nested_access() {
        initialize();

        #[derive(JsStruct)]
        struct Pair {
            pub x: i32,
            pub y: i32,
        }

        #[js_impl]
        impl Pair {
            pub fn new(x: i32, y: i32) -> Self {
                Self { x, y }
            }
        }

        #[derive(JsStruct)]
        struct PairPair {
            pub x: RcObject<Pair>,
            pub y: RcObject<Pair>,
        }

        #[js_impl]
        impl PairPair {
            pub fn new(x: RcObject<Pair>, y: RcObject<Pair>) -> Self {
                Self { x, y }
            }
        }

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let mut global_scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = build_context(&mut global_scope, |context_builder| {
            module_add_class!(context_builder, Pair);
            module_add_class!(context_builder, PairPair);
        });

        assert_eq!(
            hoo_meta_context.evaluate_script_get_string(
                "let p = new Pair(1, 2); let dp = new PairPair(p, p); dp.x.x = 3; dp.y.x"
            ),
            "3"
        );
    }

    #[test]
    fn garbage_collection() {
        initialize();

        #[derive(JsStruct)]
        struct Pair {
            pub x: i32,
            pub y: i32,
        }

        #[js_impl]
        impl Pair {
            pub fn new(x: i32, y: i32) -> Self {
                Self { x, y }
            }
        }

        #[js_function]
        fn empty_pair() -> RcObject<Pair> {
            return RcObject::new(Pair::new(0, 0));
        }

        static mut COUNT: i32 = 0;
        unsafe { COUNT = 0 };

        impl Drop for Pair {
            fn drop(&mut self) {
                unsafe {
                    COUNT += 1;
                }
            }
        }

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        let mut global_scope = v8::HandleScope::new(isolate);

        let mut hoo_meta_context = build_context(&mut global_scope, |context_builder| {
            module_add_function!(context_builder, empty_pair);
            module_add_class!(context_builder, Pair);
        });

        assert_eq!(
            hoo_meta_context.evaluate_script_get_string(
                "function foo1() { let x = new Pair(1, 2); return x.x; } foo1()"
            ),
            "1"
        );

        assert_eq!(unsafe { COUNT }, 0);

        hoo_meta_context
            .scope_mut()
            .request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);

        assert_eq!(unsafe { COUNT }, 1);

        assert_eq!(
            hoo_meta_context.evaluate_script_get_string(
                "function foo2() { let x = empty_pair(); return x.x; } foo2()"
            ),
            "0"
        );

        assert_eq!(unsafe { COUNT }, 1);

        hoo_meta_context
            .scope_mut()
            .request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);

        assert_eq!(unsafe { COUNT }, 2);

        assert_eq!(hoo_meta_context.evaluate_script_get_string("0"), "0");
        assert_eq!(unsafe { COUNT }, 2);
    }
}
