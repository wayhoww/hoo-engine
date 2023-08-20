mod device;
mod editor;
mod global;
mod graphics;
mod io;
mod object;
mod utils;

use editor::FEditor;
use global::{configs::Configs, resources::FGlobalResources};
use hoo_object::RcObject;
use object::context::HContext;
use utils::RcMut;

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

    // editor
    editor: RefCell<FEditor>,
    egui_platform: RefCell<egui_winit_platform::Platform>,

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
    pub async fn new_async(window: &RcMut<winit::window::Window>) -> Rc<RefCell<HooEngine>> {
        let window_ref = window.borrow();
        let platform =
            egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
                physical_width: window_ref.inner_size().width,
                physical_height: window_ref.inner_size().height,
                scale_factor: window_ref.scale_factor(),
                font_definitions: egui::FontDefinitions::default(),
                style: Default::default(),
            });
        let platform = RefCell::new(platform);
        let renderer = RefCell::new(Renderer::new_async(window).await);
        let out = HooEngine {
            configs: Configs {
                resources_path: "resources".into(),
            },
            renderer: renderer,
            resources: RefCell::new(FGlobalResources::new()),
            editor: RefCell::new(FEditor::new()),
            egui_platform: platform,
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

    pub fn get_editor(&self) -> Ref<FEditor> {
        self.editor.borrow()
    }

    pub fn get_editor_mut(&self) -> RefMut<FEditor> {
        self.editor.borrow_mut()
    }

    pub fn get_egui_platform(&self) -> Ref<egui_winit_platform::Platform> {
        self.egui_platform.borrow()
    }

    pub fn get_egui_platform_mut(&self) -> RefMut<egui_winit_platform::Platform> {
        self.egui_platform.borrow_mut()
    }

    pub fn receive_event(&self, event: &winit::event::Event<()>) {
        self.egui_platform.borrow_mut().handle_event(event);
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
