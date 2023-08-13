mod bundle;
mod device;
mod editor;
mod global;
mod graphics;
mod io;
mod object;
mod utils;

use global::resources::FGlobalResources;

use std::{
    cell::{Ref, RefCell, RefMut},
    mem::MaybeUninit,
    rc::Rc,
};

use crate::graphics::Renderer;

pub struct HooEngine {
    // graphics
    renderer: RefCell<Renderer>,
    resources: RefCell<FGlobalResources>,
}

thread_local! {
    static HOO_ENGINE: RefCell<Option<Rc<RefCell<HooEngine>>>> = RefCell::new(None);
}

pub fn hoo_engine() -> Rc<RefCell<HooEngine>> {
    HOO_ENGINE.with(|e| e.borrow().as_ref().unwrap().clone())
}

pub fn initialize_hoo_engine(engine: Rc<RefCell<HooEngine>>) {
    HOO_ENGINE.with(|e| {
        assert!(e.borrow().is_none());
        *e.borrow_mut() = Some(engine);
    });
}

impl HooEngine {
    pub async fn new_async(window: &winit::window::Window) -> Rc<RefCell<HooEngine>> {
        let out = HooEngine {
            // assume: do not access HooEngine::renderer in init of Renderer
            #[allow(invalid_value)]
            renderer: RefCell::new(Renderer::new_async(window).await),
            resources: RefCell::new(FGlobalResources::new()),
        };
        rcmut!(out)
    }

    pub fn next_frame(&self) {
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

// #[wasm_bindgen]
// pub struct JsHooEngine {
//     engine: Rc<RefCell<HooEngine>>,
// }

// #[wasm_bindgen]
// impl JsHooEngine {
//     pub async fn new_async(context: web_sys::GpuCanvasContext) -> Self {
//         let engine = HooEngine::new_async(context).await;
//         Self { engine }
//     }

//     pub fn next_frame(&mut self) {
//         self.engine.borrow().next_frame();
//     }
// }
