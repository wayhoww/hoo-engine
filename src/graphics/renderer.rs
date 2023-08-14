use crate::device::graphics::*;
use crate::*;

use super::pipeline::FGraphicsPipeline;
use super::FGraphicsContext;

pub struct Renderer {
    // // resources
    graphics_encoder: FDeviceEncoder,
    graphics_pipeline: Option<FGraphicsPipeline>,
    graphics_context: FGraphicsContext,
    // uniform_buffer: FBuffer,
    // model: Option<FModel>,
    // pass: Option<FPass>,
}

impl Renderer {
    pub async fn new_async(window: &winit::window::Window) -> Self {
        let graphics_encoder = FDeviceEncoder::new_async(window).await;

        Self {
            graphics_encoder,
            graphics_pipeline: None,
            graphics_context: FGraphicsContext::new(),
        }
    }

    pub async fn initialize_test_resources(&mut self) {
        let graphics_pipeline = FGraphicsPipeline::new_async(&self.graphics_encoder).await;
        self.graphics_pipeline = Some(graphics_pipeline);
    }

    pub fn get_graphics_context(&mut self) -> &mut FGraphicsContext {
        &mut self.graphics_context
    }

    // TODO: 定义几个时间点，如 begin_frame, begin_object_tick, begin_render_tick, end_render_tick, end_object_tick, end_frame
    pub fn next_frame(&mut self) {
        self.graphics_pipeline
            .as_mut()
            .unwrap()
            .draw(&mut self.graphics_encoder, &mut self.graphics_context);
        self.graphics_context.next_frame();
    }
}
