use global::resources::FGlobalResources;
use renderer::resource::FModel;
use std::{
    cell::{Ref, RefCell, RefMut},
    mem::MaybeUninit,
    panic,
    rc::Rc,
};
use wasm_bindgen::prelude::*;

use crate::{
    editor::importer::load_gltf_from_slice,
    renderer::{renderer::Renderer, resource::TRenderObject},
};

mod bundle;
mod editor;
mod global;
mod io;
mod renderer;
mod utils;

#[wasm_bindgen(start)]
fn initialize() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

pub struct HooEngine {
    // graphics
    renderer: RefCell<Renderer>,
    resources: RefCell<FGlobalResources>,
}

// 为啥要套一个 Rc? 有些地方可能要存储
// 为啥要套一个 &? 减少拷贝
// 暂时不考虑多线程
type HooEngineRef<'a> = &'a std::rc::Weak<RefCell<HooEngine>>;
type HooEngineWeak = std::rc::Weak<RefCell<HooEngine>>;

impl HooEngine {
    async fn new_async(context: web_sys::GpuCanvasContext) -> Rc<RefCell<HooEngine>> {
        let out = HooEngine {
            // assume: do not access HooEngine::renderer in init of Renderer
            #[allow(invalid_value)]
            renderer: RefCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
            resources: RefCell::new(FGlobalResources::new()),
        };
        let out_ref = rcmut!(out);
        let renderer = Renderer::new_async(&Rc::downgrade(&out_ref), context).await;

        unsafe {
            std::ptr::copy_nonoverlapping(&renderer, out_ref.borrow_mut().renderer.as_ptr(), 1)
        }

        std::mem::forget(renderer);

        out_ref
            .borrow()
            .get_renderer_mut()
            .initialize_test_resources()
            .await;

        return out_ref;
    }

    fn next_frame(&self) {
        self.renderer.borrow_mut().next_frame();
    }

    pub fn get_renderer(&self) -> Ref<Renderer> {
        self.renderer.borrow()
    }

    pub fn get_renderer_mut(&self) -> RefMut<Renderer> {
        self.renderer.borrow_mut()
    }

    pub fn get_resources(&self) -> Ref<FGlobalResources> {
        self.resources.borrow()
    }

    pub fn get_resources_mut(&self) -> RefMut<FGlobalResources> {
        self.resources.borrow_mut()
    }
}

#[wasm_bindgen]
pub struct JsHooEngine {
    engine: Rc<RefCell<HooEngine>>,
}

#[wasm_bindgen]
impl JsHooEngine {
    pub async fn new_async(context: web_sys::GpuCanvasContext) -> Self {
        let engine = HooEngine::new_async(context).await;
        Self { engine }
    }

    pub fn next_frame(&mut self) {
        self.engine.borrow().next_frame();
    }
}
