use crate::traits::*;
use crate::types::*;

use hoo_object::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;

// TODO: get rid of ALL static variables
thread_local! {
    static ID2RES: RefCell<HashMap<ObjectId, (RcAny, v8::Weak<v8::Object>)>> = RefCell::new(HashMap::new());
}

pub fn register_object(rs_val: RcAny, js_val: v8::Weak<v8::Object>) {
    ID2RES.with(|map| {
        let mut map = map.borrow_mut();
        map.insert(rs_val.id(), (rs_val, js_val));
    });
}

pub fn get_registered_rust_object(id: ObjectId) -> Option<RcAny> {
    ID2RES.with(|map| {
        let map = map.borrow();
        if let Some((rs_val, _)) = map.get(&id) {
            Some(rs_val.clone())
        } else {
            None
        }
    })
}

pub fn unregister_object(id: ObjectId) {
    ID2RES.with(|map| {
        let mut map = map.borrow_mut();
        map.remove(&id);
    });
}

impl<T: InitializeProperties> GetJsValue for RcObject<T> {
    // into 这个名字也不好
    fn get_js_value<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::Value>, JsException> {
        // 获取里面存的 js value
        let js_value = ID2RES.with(|map| {
            let map = map.borrow_mut();
            if let Some((_, js_value)) = map.get(&self.id()) {
                if let Some(js_value) = js_value.to_local(scope) {
                    return Some(js_value);
                }
            }
            None
        });

        if let Some(js_value) = js_value {
            // 如果里面确实有存 js value：最好
            Ok(js_value.into())
        } else {
            // Rust 对象还在，且下面的代码大概率不会失败
            // 因此不用清理 weak 指针

            let object = T::initialize_properties(scope);
            object.set_internal_field(0, v8::External::new(scope, self.id().to_ptr() as *mut c_void).into());

            // 这边是从既有 Rust 对象创建 Js 对象
            // TODO: GC 互通
            ID2RES.with(|map| {
                let mut map = map.borrow_mut();
                map.insert(
                    self.id(),
                    (
                        self.clone().into_any(),
                        v8::Weak::new(scope, object),
                    ),
                );
            });

            Ok(object.into())
        }
    }
}

pub fn get_external_internal_value_from_js_object<'a>(
    scope: &mut v8::HandleScope<'a>,
    object: &v8::Local<'a, v8::Object>,
    index: usize,
) -> Result<*mut c_void, TryFromJsValueError> {
    let external = object
        .get_internal_field(scope, index)
        .ok_or(TryFromJsValueError::new(&format!(
            "no internal field {}",
            index
        )))?;

    let external = v8::Local::<v8::External>::try_from(external).or(Err(
        TryFromJsValueError::new(&format!("internal field {} is not external", index)),
    ))?;

    let ptr = external.value() as *mut c_void;
    Ok(ptr)
}

impl<T: TryFromJsValue> TryFromJsValue for RcObject<T> {
    fn try_from<'a>(
        scope: &mut v8::HandleScope<'a>,
        val: &v8::Local<'a, v8::Value>,
    ) -> Result<Self, TryFromJsValueError> {
        // 首先，得是个 object
        let obj = val
            .to_object(scope)
            .ok_or(TryFromJsValueError::new("not a object"))?;

        // 而后，RcObject 对应的 JsObject 应当有三个 Internal Field
        let internal_field_count = obj.internal_field_count();
        if internal_field_count != 1 {
            return Err(TryFromJsValueError::new(&format!(
                "internal_field_count != 1, got {}",
                internal_field_count
            )));
        }

        let internal0 = get_external_internal_value_from_js_object(scope, &obj, 0)?;

        ID2RES.with(|map| {
            let map = map.borrow();
            let obj_id = ObjectId::from_ptr(internal0 as *mut c_void);

            if let Some((rs, _)) = map.get(&obj_id) {
                let obj: Result<RcObject<T>, _> = rs.clone().try_downcast();
                if let Ok(obj) = obj {
                    return Ok(obj);
                } else {
                    return Err(TryFromJsValueError::new(&format!(
                        "downcast to RcObject<T> failed, obj_id: {:?}",
                        obj_id
                    )));
                }
            }
            Err(TryFromJsValueError::new(&format!(
                "no object with id {:?}",
                obj_id
            )))
        })
    }
}
