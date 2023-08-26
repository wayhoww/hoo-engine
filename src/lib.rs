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
    rc::Rc, ops::Deref,
};

use crate::graphics::Renderer;

pub struct HooEngine {
    configs: Configs,

    // graphics
    renderer: RefCell<Renderer>,
    resources: RefCell<FGlobalResources>,

    // editor
    editor: RefCell<FEditor>,
    egui_context: RefCell<egui::Context>,
    egui_winit_state: RefCell<egui_winit::State>,

    // window
    window: RcMut<winit::window::Window>,

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

        let mut egui_winit_state = egui_winit::State::new(window_ref.deref());
        egui_winit_state.set_pixels_per_point(window_ref.scale_factor() as f32);

        let context = egui::Context::default();
        let context = RefCell::new(context);

        let renderer = RefCell::new(Renderer::new_async(window).await);
        let out = HooEngine {
            configs: Configs {
                resources_path: "resources".into(),
            },
            renderer: renderer,
            resources: RefCell::new(FGlobalResources::new()),
            editor: RefCell::new(FEditor::new()),
            egui_context: context,
            egui_winit_state: RefCell::new(egui_winit_state),
            object_context: RcObject::new(HContext::new()),
            window: window.clone()
        };
        rcmut!(out)
    }

    // HooEngine 要避免 borrow_mut. 不然就是个全局锁了

    // called before the first frame, but after HooEngine being fully constructed and singleton being initialized
    pub fn prepare(&self) {
        self.renderer.borrow_mut().prepare();
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

    pub fn get_egui_context(&self) -> Ref<egui::Context> {
        self.egui_context.borrow()
    }

    pub fn get_egui_context_mut(&self) -> RefMut<egui::Context> {
        self.egui_context.borrow_mut()
    }

    // pub fn get_egui_winit_state(&self) -> Ref<egui_winit::State> {
    //     self.egui_winit_state.borrow()
    // }

    // pub fn get_egui_winit_state_mut(&self) -> RefMut<egui_winit::State> {
    //     self.egui_winit_state.borrow_mut()
    // }

    pub fn take_egui_input(&self) -> egui::RawInput {
        self.egui_winit_state.borrow_mut().take_egui_input(self.window.borrow().deref())
    }

    pub fn receive_event(&self, event: &winit::event::Event<()>) {

        match event {
            winit::event::Event::WindowEvent { window_id: _, event } => {
                let _ = self.egui_winit_state.borrow_mut().on_event(&self.get_egui_context(), event);
            },
            _ => {}
        }
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
