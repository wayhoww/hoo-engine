mod device;
mod editor;
mod global;
mod graphics;
mod io;
mod object;
mod utils;

use global::{configs::Configs, resources::FGlobalResources};
use hoo_object::RcObject;
use object::context::HContext;

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::graphics::Renderer;

pub struct HooEngine {
    configs: Configs,

    // graphics
    renderer: RefCell<Renderer>,
    resources: RefCell<FGlobalResources>,

    object_context: RcObject<HContext>,
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
            configs: Configs {
                resources_path: "resources".into(),
            },
            renderer: RefCell::new(Renderer::new_async(window).await),
            resources: RefCell::new(FGlobalResources::new()),
            object_context: RcObject::new(HContext::new()),
        };
        rcmut!(out)
    }

    // HooEngine 要避免 borrow_mut. 不然就是个全局锁了

    // called before the first frame, but after HooEngine being fully constructed and singleton being initialized
    pub fn prepare(&self) {
        self.object_context.borrow_mut().create_demo_space();
    }

    pub fn next_frame(&self) {
        self.object_context.borrow_mut().tick(0.0);
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

    pub fn get_configs(&self) -> &Configs {
        &self.configs
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
