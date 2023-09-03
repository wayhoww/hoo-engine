use crate::*;
use crate::{device::graphics::*, object::objects::HCameraTarget};

use super::{FGraphicsContext, FPipelineContextData, FPipelineContext};

pub struct Renderer {
    // // resources
    graphics_encoder: FDeviceEncoder,
    graphics_context: RefCell<FGraphicsContext>,
    // uniform_buffer: FBuffer,
    // model: Option<FModel>,
    // pass: Option<FGraphicsPass>,
    main_viewport_texture: Option<RcMut<FTexture>>,
}

impl Renderer {
    pub async fn new_async(window: &RcMut<winit::window::Window>) -> Self {
        let graphics_encoder = FDeviceEncoder::new_async(window).await;

        Self {
            graphics_encoder,
            graphics_context: RefCell::new(FGraphicsContext::new()),
            main_viewport_texture: None,
        }
    }

    pub async fn initialize_test_resources(&mut self) {
        // do nothing
    }

    pub fn prepare(&mut self) {
        self.graphics_encoder.prepare();
    }

    pub fn submit_pipeline(&self, pipeline_context: RcMut<FPipelineContext>) {
        self.graphics_context
            .borrow_mut()
            .submit_pipeline(pipeline_context);
    }

    // TODO: 定义几个时间点，如 begin_frame, begin_object_tick, begin_render_tick, end_render_tick, end_object_tick, end_frame
    pub fn next_frame(&mut self) {
        let overlay_mode = hoo_engine().borrow().get_editor().get_state().overlay_mode;
        if !overlay_mode && self.main_viewport_texture.is_none() {
            let tex = FTexture::new_and_manage(
                ETextureFormat::Bgra8UnormSrgb,
                BTextureUsages::Attachment | BTextureUsages::Sampled,
            );
            tex.borrow_mut().set_size((800, 600));
            self.main_viewport_texture = Some(tex);
        }
        if overlay_mode && self.main_viewport_texture.is_some() {
            let _ = self.main_viewport_texture.take();
        }
        // 主动设置好还是让 editor 来获取好？  通常是 editor 来主动获取
        hoo_engine().borrow().get_editor_mut().main_viewport_texture =
            self.main_viewport_texture.clone();

        self.graphics_context
            .borrow_mut()
            .encode(&mut self.graphics_encoder);
        *self.graphics_context.borrow_mut() = FGraphicsContext::new();
    }

    pub fn get_swapchain_size(&self) -> (u32, u32) {
        self.graphics_encoder.get_swapchain_size()
    }

    pub fn get_editor_renderer_ref(&self) -> Ref<FEditorRenderer> {
        self.graphics_encoder.get_editor_renderer_ref()
    }

    pub fn get_editor_renderer_mut(&self) -> RefMut<FEditorRenderer> {
        self.graphics_encoder.get_editor_renderer_mut()
    }

    pub fn get_main_viewport_target(&self) -> HCameraTarget {
        if let Some(tex) = &self.main_viewport_texture {
            HCameraTarget::Texture(tex.clone())
        } else {
            HCameraTarget::Screen
        }
    }
}
