// 作为渲染管线（pipeline.rs） 和 游戏逻辑 (objects, 尤其是 systems) 的桥梁
// 比如，systems 往这里设置模型，渲染管线从这里获取模型
// 这一层的必要性： 暂时没有太大必要。长远来说，强迫 pipeline 暴露很多接口不合适。尤其 Rust 不支持子类。

use nalgebra_glm as glm;

use crate::{
    device::graphics::{FDeviceEncoder, FFrameEncoder, FRenderObject},
    hoo_engine,
    object::objects::{FShaderLight, HCameraTarget},
};

use super::FGraphicsPipeline;

pub struct FGraphicsContext {
    pipelines: Vec<FPipelineContext>,
}

#[derive(Default, Clone)]
pub struct FPipelineContext {
    pub camera_projection: glm::Mat4,
    pub camera_transform: glm::Mat4,

    pub render_objects: Vec<FRenderObject>,
    pub lights: Vec<FShaderLight>,

    pub render_target: HCameraTarget,
    pub render_target_size: (u32, u32),
}

impl FGraphicsContext {
    pub fn new() -> Self {
        Self { pipelines: vec![] }
    }

    pub fn submit_pipeline(&mut self, pipeline_context: FPipelineContext) {
        self.pipelines.push(pipeline_context);
    }

    pub fn encode(&mut self, encoder: &mut FDeviceEncoder) {
        let mut pipeline_encoders = vec![]; // TODO: 支持不同种类的 pipeline
        for ctx in self.pipelines.iter_mut() {
            let mut pipeline_encoder = FGraphicsPipeline::new();
            pipeline_encoder.prepare(ctx);
            pipeline_encoders.push(pipeline_encoder);
        }

        encoder.encode_frame(|frame_encoder| {
            for pipeline in self.pipelines.iter_mut().zip(pipeline_encoders.iter_mut()) {
                let (pipeline_context, pipeline_encoder) = pipeline;
                pipeline_encoder.draw(frame_encoder, pipeline_context);
            }
        });

        self.pipelines.clear()
    }
}

impl FPipelineContext {
    pub fn new() -> Self {
        FPipelineContext {
            camera_projection: glm::perspective(1920.0 / 1080.0, 45.0, 0.1, 1000.0),
            camera_transform: glm::look_at(
                &glm::vec3(3.0, 3.0, 3.0),
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            ),
            render_objects: Vec::new(),
            lights: Vec::new(),
            render_target: HCameraTarget::default(),
            render_target_size: (0, 0),
        }
    }

    pub fn add_render_object(&mut self, render_object: FRenderObject) {
        self.render_objects.push(render_object);
    }

    pub fn set_camera_transform(&mut self, camera_transform: glm::Mat4) {
        self.camera_transform = camera_transform;
    }

    pub fn set_camera_projection(&mut self, camera_projection: glm::Mat4) {
        self.camera_projection = camera_projection;
    }

    pub fn set_lights(&mut self, lights: Vec<FShaderLight>) {
        self.lights = lights;
    }

    pub fn set_render_target(&mut self, render_target: HCameraTarget) {
        self.render_target = render_target;
        match &self.render_target {
            HCameraTarget::Texture(tex) => {
                let size = tex.borrow().size();
                self.render_target_size = size;
            }
            HCameraTarget::Screen => {
                self.render_target_size = hoo_engine().borrow().get_renderer().get_swapchain_size();
            }
        }
    }
}
