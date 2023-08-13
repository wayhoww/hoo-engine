use crate::device::graphics::*;
use crate::*;

use super::pipeline::FGraphicsPipeline;

pub struct Renderer {
    // // resources
    graphics_encoder: FDeviceEncoder,
    graphics_pipeline: Option<FGraphicsPipeline>,
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
        }
    }

    pub async fn initialize_test_resources(&mut self) {
        let graphics_pipeline = FGraphicsPipeline::new_async(&self.graphics_encoder).await;
        self.graphics_pipeline = Some(graphics_pipeline);
    }

    pub fn next_frame(&mut self) {
        self.graphics_pipeline
            .as_mut()
            .unwrap()
            .draw(&mut self.graphics_encoder);
    }
}
