use crate::*;

use super::encoder::*;
use super::pipeline::FGraphicsPipeline;
use super::resource::*;

use nalgebra_glm as glm;

pub struct Renderer {
    hoo_engine: HooEngineWeak,

    // // resources
    graphics_encoder: FDeviceEncoder,
    graphics_pipeline: Option<FGraphicsPipeline>,
    // uniform_buffer: FBuffer,
    // model: Option<FModel>,
    // pass: Option<FPass>,
}

impl Renderer {
    pub async fn new_async<'a>(h: HooEngineRef<'a>, window: &winit::window::Window) -> Self {
        let graphics_encoder = FDeviceEncoder::new_async(h, window).await;

        Self {
            hoo_engine: h.clone(),
            graphics_encoder,
            graphics_pipeline: None,
        }
    }

    pub async fn initialize_test_resources(&mut self) {
        let graphics_pipeline =
            FGraphicsPipeline::new_async(&self.hoo_engine, &self.graphics_encoder).await;
        self.graphics_pipeline = Some(graphics_pipeline);
    }

    pub fn next_frame(&mut self) {
        self.graphics_pipeline
            .as_mut()
            .unwrap()
            .draw(&mut self.graphics_encoder);
    }
}
