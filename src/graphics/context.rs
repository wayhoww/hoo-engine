// 作为渲染管线（pipeline.rs） 和 游戏逻辑 (objects, 尤其是 systems) 的桥梁
// 比如，systems 往这里设置模型，渲染管线从这里获取模型
// 这一层的必要性： 暂时没有太大必要。长远来说，强迫 pipeline 暴露很多接口不合适。尤其 Rust 不支持子类。

use nalgebra_glm as glm;

use crate::{
    device::graphics::{FDeviceEncoder, FModel, FRenderObject},
    hoo_engine,
};

use super::FGraphicsPipeline;

pub struct FGraphicsContext {
    pipelines: Vec<FPipelineContext>,
}

pub struct FPipelineContext {
    pub camera_projection: glm::Mat4,
    pub camera_transform: glm::Mat4,
    pub render_objects: Vec<FRenderObject>,
}

impl FGraphicsContext {
    pub fn new() -> Self {
        return Self { pipelines: vec![] };
    }

    pub fn submit_pipeline(&mut self, pipeline_context: FPipelineContext) {
        self.pipelines.push(pipeline_context);
    }

    pub fn encode(&mut self, encoder: &mut FDeviceEncoder) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.encode(encoder)
        }

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

    pub fn encode(&mut self, encoder: &mut FDeviceEncoder) {
        let mut pipeline = FGraphicsPipeline::new(encoder);
        pipeline.draw(encoder, self);
    }
}
