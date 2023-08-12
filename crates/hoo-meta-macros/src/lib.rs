use quote::{quote, TokenStreamExt};
use std::vec;

// 生命周期坚决不支持，可以报 warning，加 attr 避免 warning
// template 至少要有限支持，用于数学库函数等
// reference 要有限支持，比如 Rc 等。函数参数是 mut 引用类型咋弄要想想

// 函数
macro_rules! js_function_rs_return_format_string {
    () => {
        "__hoo_meta_js_function_{}"
    };
}

macro_rules! js_function_format_string {
    () => {
        "__hoo_meta_js_function_callback_{}"
    };
}

macro_rules! js_function_name_format_string {
    () => {
        "__hoo_meta_js_function_name_{}"
    };
}

#[proc_macro_attribute]
pub fn js_impl(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2: proc_macro2::TokenStream = item.into();
    let syn_item = syn::parse2::<syn::Item>(item2.clone()).unwrap();

    let mut funcs = Vec::new();

    if let syn::Item::Impl(syn_impl) = syn_item {
        let impl_type = syn_impl.self_ty;

        let mut method_names: Vec<syn::Ident> = Vec::new();

        for item in syn_impl.items {
            match item {
                syn::ImplItem::Fn(impl_fn) => {
                    if let syn::Visibility::Public(_) = impl_fn.vis {
                        let signature = impl_fn.sig.clone();
                        let name = signature.ident;
                        let mut is_method: bool = false;

                        // 这里应该要做更多检查，比如是不是只有一个有 self 等
                        for (index, input) in signature.inputs.iter().enumerate() {
                            match input {
                                syn::FnArg::Receiver(_) => {
                                    if index == 0 {
                                        is_method = true;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if is_method {
                            method_names.push(name.clone());
                        }

                        let generated = generate_js_function(impl_fn.clone(), true);
                        funcs.push(generated);
                    }
                }
                _ => {}
            }
        }

        let method_bindings = method_names.iter().map(|method_ident| {
            let underlying_func_ident = syn::Ident::new(
                &format!(
                    js_function_rs_return_format_string!(),
                    method_ident.to_string()
                ),
                method_ident.span(),
            );

            let method_name_str = method_ident.to_string();

            quote!(
                {
                    fn function<'a, 's, 'b> (
                        scope: &'a mut v8::HandleScope<'s>,
                        args: v8::FunctionCallbackArguments<'s>,
                        mut retval: v8::ReturnValue<'b>,
                    ) {
                        let result =  #impl_type::#underlying_func_ident(scope, args);

                        match result {
                            Ok(result) => {
                                let result = hoo_meta::GetJsValue::get_js_value(&result, scope);
                                match result {
                                    Err(e) => {
                                        let exception_msg = v8::String::new(scope, &e.error_message()).unwrap().into();
                                        let exception = v8::Exception::error(scope, exception_msg);
                                        scope.throw_exception(exception);
                                    },
                                    Ok(result) => {
                                        retval.set(result);
                                    }
                                }
                            },
                            Err(err) => {
                                let exception_msg = v8::String::new(scope, &err.error_message()).unwrap().into();
                                let exception = v8::Exception::error(scope, exception_msg);
                                scope.throw_exception(exception);
                            }
                        }
                    }

                    let function_template = v8::FunctionTemplate::new(scope, function);
                    let function = function_template.get_function(scope).unwrap();
                    let function_name = v8::String::new(scope, #method_name_str).unwrap();
                    object.set(scope, function_name.into(), function.into());
                }
            )
        });

        let mut generated = quote!(
            impl #impl_type {
                #(#funcs)*

                fn __hoo_meta_set_trait_methods(
                    scope: &mut v8::HandleScope,
                    object: v8::Local<v8::Object>,
                ) {
                    #(#method_bindings)*
                }
            }

        );

        generated.append_all(item2);
        generated.into()
    } else {
        panic!("js_impl attribute can only be applied to impl blocks");
    }
}

#[proc_macro_derive(JsStruct)]
pub fn js_struct_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2: proc_macro2::TokenStream = item.clone().into();
    let syn_item = syn::parse2::<syn::Item>(item2.clone()).unwrap();
    if let syn::Item::Struct(syn_struct) = syn_item {
        let mut out = quote!();
        out.append_all(get_item_struct_converter(&syn_struct));
        out.append_all(get_getters_setters_ctor(&syn_struct));
        out.into()
    } else {
        panic!("js_struct attribute can only be applied to structs");
    }
}

#[proc_macro_derive(JsStructNoConstructor)]
pub fn js_struct_no_constructor_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2: proc_macro2::TokenStream = item.clone().into();
    let syn_item = syn::parse2::<syn::Item>(item2.clone()).unwrap();
    if let syn::Item::Struct(syn_struct) = syn_item {
        let mut out = quote!();
        out.append_all(get_item_struct_converter(&syn_struct));
        out.into()
    } else {
        panic!("js_struct attribute can only be applied to structs");
    }
}

fn get_getters_setters_ctor(st: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let ident = &st.ident;
    let struct_name = ident.to_string();

    let mut fields: Vec<(syn::Ident, syn::Type)> = vec![];

    for field in &st.fields {
        if let Some(field_ident) = &field.ident {
            if let syn::Visibility::Public(_) = field.vis {
                let field_type = field.ty.clone();
                fields.push((field_ident.clone(), field_type));
            }
            // else skip private fields
        } else {
            panic!("only supports structs with named fields");
        }
    }

    let converter_func_ident = syn::Ident::new(
        &format!(js_function_rs_return_format_string!(), "new"),
        proc_macro2::Span::call_site(),
    );

    let getters_setters = fields.iter().map(|(key_ident, ty)| {
        let key_str = key_ident.to_string();

        quote!(
            {
                // getters && setters
                fn getter<'s>(
                    scope: &mut v8::HandleScope<'s>,
                    name: v8::Local<'s, v8::Name>,
                    args: v8::PropertyCallbackArguments<'s>,
                    mut retval: v8::ReturnValue,
                ) {
                    let this = args.this();
                    // 能访问到这个 getter/setter, 说明一定是有效的
                    let rsobj_index = hoo_meta::get_external_internal_value_from_js_object(scope, &this, 0).unwrap();
                    let rsobj = hoo_meta::get_registered_rust_object(hoo_object::ObjectId::from_ptr(rsobj_index as *const std::os::raw::c_void)).unwrap();

                    let rsobj = rsobj.try_downcast::<#ident>().unwrap();
                    let jsobj = hoo_meta::GetJsValue::get_js_value(&rsobj.borrow().#key_ident, scope);

                    match jsobj {
                        Ok(jsobj) => {
                            retval.set(jsobj);
                        }
                        Err(err) => {
                            println!("Failed to convert to js value: {}", &err.error_message());
                        }
                    }
                }

                fn setter<'s>(
                    scope: &mut v8::HandleScope<'s>,
                    name: v8::Local<'s, v8::Name>,
                    value: v8::Local<'s, v8::Value>,
                    args: v8::PropertyCallbackArguments<'s>,
                    mut retval: v8::ReturnValue,
                ) {
                    let val: Result<#ty, _> = hoo_meta::TryFromJsValue::try_from(scope, &value);
                    match val {
                        Ok(val) => {
                            let this = args.this();
                            let rsobj_index = hoo_meta::get_external_internal_value_from_js_object(scope, &this, 0).unwrap();
                            let rsobj = hoo_meta::get_registered_rust_object(hoo_object::ObjectId::from_ptr(rsobj_index as *const std::os::raw::c_void)).unwrap();

                            let rsobj = rsobj.try_downcast::<#ident>().unwrap();
                            rsobj.borrow_mut().#key_ident = val;
                        }
                        Err(err) => {
                            println!("Failed to convert to rust value: {}", &err.error_message());
                        }
                    }
                }

                let key = v8::String::new(scope, #key_str).unwrap();
                this.set_accessor_with_setter(scope, key.into(), getter, setter);
            }
        )
    });

    let getters_setters_clone = getters_setters.clone();

    let generated = quote!(
        impl #ident {
            pub fn __hoo_meta_register_struct<'s, 'a>(
                module_builder: &mut impl hoo_meta::ModuleLikeBuilder<'s, 'a>
            ) {
                fn new<'a, 's, 'b>(
                    scope: &'a mut v8::HandleScope<'s>,
                    args: v8::FunctionCallbackArguments<'s>,
                    mut retval: v8::ReturnValue<'b>,
                ) {
                    let this = args.this();
                    if this.is_null_or_undefined() {
                        return;
                    }

                    // TODO: 怎么挪到外面去？
                    let instance_template = v8::ObjectTemplate::new(scope);
                    instance_template.set_internal_field_count(1);

                    // 覆盖 this，增加 internal field
                    let this = instance_template.new_instance(scope).unwrap();
                    retval.set(this.into());

                    // 构造 RcObject
                    let rs_stu: #ident = #ident::#converter_func_ident(scope, args).unwrap();
                    let rs_obj = hoo_object::RcObject::new(rs_stu);

                    hoo_meta::register_object_enabling_bigc(scope, rs_obj.into_any(), this);

                    #(#getters_setters)*

                    #ident::__hoo_meta_set_trait_methods(scope, this);
                };

                // todo: cache?
                let scope = module_builder.get_global_scope();
                let function_template = v8::FunctionTemplate::new(scope, new);
                let entryname = v8::String::new(scope, #struct_name).unwrap();
                module_builder.get_template().set(entryname.into(), function_template.into());
            }
        }

        impl hoo_meta::BindProperties for #ident {
            fn bind_properties<'a>(
                scope: &mut v8::HandleScope<'a>,
            ) -> v8::Local<'a, v8::Object> {
                let instance_template = v8::ObjectTemplate::new(scope);
                instance_template.set_internal_field_count(1);

                let this = instance_template.new_instance(scope).unwrap();
                #(#getters_setters_clone)*

                #ident::__hoo_meta_set_trait_methods(scope, this);

                this
            }
        }
    );

    generated
}

// TODO：访问权限控制有一些问题：如果不是所有字段都是 pub 的，那么不应当允许从 Js 到 Rust 的转换
// 类型转换
fn get_item_struct_converter(st: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let ident = &st.ident;

    let mut fields: Vec<(syn::Ident, syn::Type)> = vec![];

    for field in &st.fields {
        if let Some(field_ident) = &field.ident {
            let field_type = field.ty.clone();
            fields.push((field_ident.clone(), field_type));
        } else {
            panic!("only supports structs with named fields");
        }
    }

    let binding_stmts = fields
        .iter()
        .map(|(field_ident, field_type)| {
            let field_name = field_ident.to_string();
            quote!(
                let key = v8::String::new(scope, #field_name).unwrap();
                let js_value = js_object.get(scope, key.into()).ok_or(hoo_meta::TryFromJsValueError::new(&format!("field does not exist: {}", #field_name)))?;
                let #field_ident = <#field_type as hoo_meta::TryFromJsValue>::try_from(scope, &js_value)?;
            )
        });

    let set_js_object_stmts = fields.iter().map(|(field_ident, _)| {
        let field_name = field_ident.to_string();
        quote!(
            let key = v8::String::new(scope, #field_name).unwrap();
            let value = hoo_meta::GetJsValue::get_js_value(&self.#field_ident, scope)?;
            js_object.set(scope, key.into(), value);
        )
    });

    let set_js_object_stmts_clone = set_js_object_stmts.clone();

    let field_idents = fields.iter().map(|(field_ident, _)| field_ident);

    let generated = quote!(
        impl hoo_meta::TryFromJsValue for #ident {
            fn try_from<'a>(scope: &mut v8::HandleScope<'a>, val: &v8::Local<'a, v8::Value>) -> Result<Self, hoo_meta::TryFromJsValueError> {
                // TODO: 如果 JsValue 是 RcObject，处理方式还能再简单点
                let js_object = val.to_object(scope).ok_or(hoo_meta::TryFromJsValueError::new("not an object"))?;
                #(#binding_stmts)*
                let out = #ident { #(#field_idents),* };
                Ok(out)
                // TODO: 这里或许需要调用一下做有效性检查的 trait
            }
        }

        impl hoo_meta::GetJsValue for #ident {
            fn get_js_value<'a>(&self, scope: &mut v8::HandleScope<'a>) -> Result<v8::Local<'a, v8::Value>, hoo_meta::JsException> {
                let js_object = v8::Object::new(scope);
                #(#set_js_object_stmts)*
                Ok(js_object.into())
            }
        }

        impl hoo_meta::FillJsObject for #ident {
            fn fill_js_object<'a, 'b>(
                &self,
                js_object: v8::Local<'b, v8::Object>,
                scope: &mut v8::HandleScope<'a>,
            ) -> Result<(), hoo_meta::JsException>
                where 'a: 'b
            {
                #(#set_js_object_stmts_clone)*
                Ok(())
            }
        }
    );

    generated
}

#[proc_macro]
pub fn get_js_function(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let iter = item.into_iter();
    assert!(iter.clone().count() == 1);

    let maybe_ident = iter.last().unwrap();

    match maybe_ident {
        proc_macro::TokenTree::Ident(ident) => {
            let new_ident = syn::Ident::new(
                &format!(js_function_format_string!(), ident),
                proc_macro2::Span::call_site(),
            );
            quote!(#new_ident).into()
        }
        _ => unimplemented!("get_js_function only supports identifiers"),
    }
}


#[proc_macro]
pub fn get_js_function_name_string(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let iter = item.into_iter();
    assert!(iter.clone().count() == 1);

    let maybe_ident = iter.last().unwrap();

    match maybe_ident {
        proc_macro::TokenTree::Ident(ident) => {
            let new_ident = syn::Ident::new(
                &format!(js_function_name_format_string!(), ident),
                proc_macro2::Span::call_site(),
            );
            quote!(#new_ident).into()
        }
        _ => unimplemented!("get_js_function only supports identifiers"),
    }
}


#[proc_macro_attribute]
pub fn js_function(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item2: proc_macro2::TokenStream = item.clone().into();
    let syn_item = syn::parse2::<syn::Item>(item2.clone()).unwrap();
    if let syn::Item::Fn(syn_fn) = syn_item {
        let mut generated = generate_js_function(syn_fn, false);
        generated.append_all(item2);
        generated.into()
    } else {
        panic!("js_method only supports functions");
    }
}

// #[proc_macro_attribute]
// pub fn js_method(
//     _attr: proc_macro::TokenStream,
//     item: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let item2: proc_macro2::TokenStream = item.clone().into();
//     let syn_item = syn::parse2::<syn::Item>(item2.clone()).unwrap();
//     if let syn::Item::Fn(syn_fn) = syn_item {
//         let mut generated = generate_js_function(syn_fn, true);
//         generated.append_all(item2);
//         generated.into()
//     } else {
//         panic!("js_method only supports functions");
//     }
// }

trait FunctionType {
    fn get_signature(&self) -> &syn::Signature;
}

impl FunctionType for syn::ItemFn {
    fn get_signature(&self) -> &syn::Signature {
        &self.sig
    }
}

impl FunctionType for syn::ImplItemFn {
    fn get_signature(&self) -> &syn::Signature {
        &self.sig
    }
}

fn generate_js_function(syn_fn: impl FunctionType, in_impl: bool) -> proc_macro2::TokenStream {
    let signature = syn_fn.get_signature().clone();
    let ident = &signature.ident;
    let ret_ty = signature.output;
    let ret_ty = match ret_ty {
        syn::ReturnType::Default => {
            quote!(())
        }
        syn::ReturnType::Type(_, dtype) => {
            quote!(#dtype)
        }
    };

    let new_ident_1 = syn::Ident::new(
        &format!(js_function_rs_return_format_string!(), ident),
        proc_macro2::Span::call_site(),
    );

    let new_ident_2 = syn::Ident::new(
        &format!(js_function_format_string!(), ident),
        proc_macro2::Span::call_site(),
    );

    let function_name_ident = syn::Ident::new(
        &format!(js_function_name_format_string!(), ident),
        proc_macro2::Span::call_site(),
    );

    let function_name_str = ident.to_string();

    // check: self is not exisits
    let arg_count = signature.inputs.len() as i32;

    let mut arguments_getter: Vec<proc_macro2::TokenStream> = vec![];

    let mut pats: Vec<proc_macro2::TokenStream> = vec![];
    let mut is_method = false;

    for (i, arg) in signature.inputs.iter().enumerate() {
        let i = i as i32;
        let pat = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
        match arg {
            syn::FnArg::Receiver(rec) => {
                // &self && &mut self.
                // &mut self 不支持非 RcObject

                if rec.reference.is_none() {
                    panic!("self must be a reference");
                }

                // let is_mut = rec.mutability.is_some();

                // let ty = rec.ty.clone();

                // 如果是 &self, 并且收到的 args.get(#i) 不是一个 RcObject 的话
                // let generated = quote!(
                //     let #pat = <Self as hoo_meta::TryFromJsValue>::try_from(scope, &args.get(#i))?;
                // );
                // arguments_getter.push(generated);
                // pats.push(quote!(& #pat));

                // 否则
                let generated = quote!(
                    let #pat = <hoo_object::RcObject<Self> as hoo_meta::TryFromJsValue>::try_from(scope, &args.this().into())?;
                );
                arguments_getter.push(generated);
                pats.push(quote!(#pat.borrow().deref()));

                if i == 0 {
                    is_method = true;
                }
            }
            syn::FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;

                let generated = quote!(
                    let #pat = <#ty as hoo_meta::TryFromJsValue>::try_from(scope, &args.get(#i - this_offset))?;
                );

                arguments_getter.push(generated);
                pats.push(quote!(#pat));
            }
        }
    }

    if !is_method {
        let qualifier = if in_impl { quote!(Self::) } else { quote!() };

        let generated = quote!(
            fn #new_ident_1<'a, 's, 'b> (
                scope: &'a mut v8::HandleScope<'s>,
                args: v8::FunctionCallbackArguments<'s>,
            ) -> Result<#ret_ty, hoo_meta::TryFromJsValueError> {
                let this_offset = 0;

                let arg_count = args.length();
                if arg_count != #arg_count {
                    return Err(hoo_meta::TryFromJsValueError::new("incorrect number of arguments"));
                }

                #(#arguments_getter)*

                let result = #qualifier #ident(#(#pats),*);
                Ok(result)
            }

            
            const #function_name_ident: &str = #function_name_str;

            fn #new_ident_2<'a, 's, 'b> (
                scope: &'a mut v8::HandleScope<'s>,
                args: v8::FunctionCallbackArguments<'s>,
                mut retval: v8::ReturnValue<'b>,
            ) {
                let result = #qualifier #new_ident_1(scope, args);

                match result {
                    Ok(result) => {
                        let result = hoo_meta::GetJsValue::get_js_value(&result, scope);
                        match result {
                            Err(e) => {
                                let exception_msg = v8::String::new(scope, &e.error_message()).unwrap().into();
                                let exception = v8::Exception::error(scope, exception_msg);
                                scope.throw_exception(exception);
                            },
                            Ok(result) => {
                                retval.set(result);
                            }
                        }
                    },
                    Err(err) => {
                        let exception_msg = v8::String::new(scope, &err.error_message()).unwrap().into();
                        let exception = v8::Exception::error(scope, exception_msg);
                        scope.throw_exception(exception);
                    }
                }
            }
        );

        generated
    } else {
        let generated = quote!(
            fn #new_ident_1<'a, 's, 'b> (
                scope: &'a mut v8::HandleScope<'s>,
                args: v8::FunctionCallbackArguments<'s>,
            ) -> Result<#ret_ty, hoo_meta::TryFromJsValueError> {
                let this_offset = 1;

                // println

                let arg_count = args.length();
                if arg_count != #arg_count - this_offset {
                    return Err(hoo_meta::TryFromJsValueError::new("incorrect number of arguments"));
                }

                #(#arguments_getter)*

                let result = Self::#ident(#(#pats),*);
                Ok(result)
            }
        );
        generated
    }
}
