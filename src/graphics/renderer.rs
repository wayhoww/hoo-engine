use crate::device::graphics::*;
use crate::*;

use super::pipeline::FGraphicsPipeline;
use super::{FGraphicsContext, FPipelineContext};

pub struct Renderer {
    // // resources
    graphics_encoder: FDeviceEncoder,
    graphics_context: RefCell<FGraphicsContext>,
    // uniform_buffer: FBuffer,
    // model: Option<FModel>,
    // pass: Option<FPass>,
}

impl Renderer {
    pub async fn new_async(window: &winit::window::Window) -> Self {
        let graphics_encoder = FDeviceEncoder::new_async(window).await;

        Self {
            graphics_encoder,
            graphics_context: RefCell::new(FGraphicsContext::new()),
        }
    }

    pub async fn initialize_test_resources(&mut self) {
        // do nothing
    }

    pub fn submit_pipeline(&self, pipeline_context: FPipelineContext) {
        self.graphics_context
            .borrow_mut()
            .submit_pipeline(pipeline_context);
    }

    // TODO: 定义几个时间点，如 begin_frame, begin_object_tick, begin_render_tick, end_render_tick, end_object_tick, end_frame
    pub fn next_frame(&mut self) {
        self.graphics_context
            .borrow_mut()
            .encode(&mut self.graphics_encoder);
        *self.graphics_context.borrow_mut() = FGraphicsContext::new();
    }
}
