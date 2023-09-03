// 作为渲染管线（pipeline.rs） 和 游戏逻辑 (objects, 尤其是 systems) 的桥梁
// 比如，systems 往这里设置模型，渲染管线从这里获取模型
// 这一层的必要性： 暂时没有太大必要。长远来说，强迫 pipeline 暴露很多接口不合适。尤其 Rust 不支持子类。

use std::{cell::RefCell, ops::DerefMut};

use nalgebra_glm as glm;

use crate::{
    device::graphics::{FDeviceEncoder, FFrameEncoder, FRenderObject},
    hoo_engine,
    object::objects::{FShaderLight, HCameraTarget},
    utils::RcMut,
};

use super::{pipeline, FGraphicsPipeline};

pub struct FGraphicsContext {
    pipelines: Vec<RcMut<FPipelineContext>>,
}

#[derive(Default, Clone)]
pub struct FPipelineContextData {
    pub camera_projection: glm::Mat4,
    pub camera_transform: glm::Mat4,

    pub render_objects: Vec<FRenderObject>,
    pub lights: Vec<FShaderLight>,

    pub render_target: HCameraTarget,
    pub render_target_size: (u32, u32),
}

pub struct FPipelineContext {
    pub data: RefCell<FPipelineContextData>,
    pub pipeline: RefCell<FGraphicsPipeline>,
}

impl FPipelineContext {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(FPipelineContextData::default()),
            pipeline: RefCell::new(FGraphicsPipeline::new()),
        }
    }
}

impl FGraphicsContext {
    pub fn new() -> Self {
        Self { pipelines: vec![] }
    }

    pub fn submit_pipeline(&mut self, pipeline_context: RcMut<FPipelineContext>) {
        self.pipelines.push(pipeline_context);
    }

    pub fn encode(&mut self, encoder: &mut FDeviceEncoder) {
        // let mut pipeline_encoders = vec![]; // TODO: 支持不同种类的 pipeline
        for pipeline in self.pipelines.iter_mut() {
            let mut pipeline = pipeline.borrow_mut();
            let mut pipeline_encoder = pipeline.pipeline.borrow_mut();
            let mut context_data = pipeline.data.borrow_mut();
            pipeline_encoder.prepare(context_data.deref_mut());
        }

        encoder.encode_frame(|frame_encoder| {
            for pipeline in self.pipelines.iter_mut() {
                let pipeline = pipeline.borrow();
                let mut pipeline_encoder = pipeline.pipeline.borrow_mut();
                let mut context_data = pipeline.data.borrow_mut();
                pipeline_encoder.draw(frame_encoder, context_data.deref_mut());

                context_data.clear();
            }
        });

        self.pipelines.clear()
    }
}

impl FPipelineContextData {
    pub fn new() -> Self {
        FPipelineContextData {
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

    pub fn clear(&mut self) {
        self.render_objects.clear();
        self.lights.clear();
    }
}
