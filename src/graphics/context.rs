// 作为渲染管线（pipeline.rs） 和 游戏逻辑 (objects, 尤其是 systems) 的桥梁
// 比如，systems 往这里设置模型，渲染管线从这里获取模型
// 这一层的必要性： 暂时没有太大必要。长远来说，强迫 pipeline 暴露很多接口不合适。尤其 Rust 不支持子类。

use nalgebra_glm as glm;

use crate::device::graphics::{FModel, FRenderObject};

pub struct FGraphicsContext {
    pub camera_projection: glm::Mat4,
    pub camera_transform: glm::Mat4,
    pub render_objects: Vec<FRenderObject>,
}

impl FGraphicsContext {
    pub fn new() -> Self {
        FGraphicsContext {
            camera_projection: glm::perspective(800.0 / 600.0, 45.0, 0.1, 100.0),
            camera_transform: glm::look_at(
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, -1.0),
                &glm::vec3(0.0, 1.0, 0.0),
            ),
            render_objects: Vec::new(),
        }
    }

    pub fn add_render_object(&mut self, render_object: FRenderObject) {
        self.render_objects.push(render_object);
    }

    pub fn next_frame(&mut self) {
        self.render_objects.clear();
    }
}
