use std::any::Any;

use hoo_object::RcObject;

use crate::{
    hoo_engine,
    object::{components::*, objects::HCamera, space::HSpace},
};

use super::{FSystemTickContext, HGraphicsSystem};

// 有实际需求的时候再考虑多相机的问题，修改不是很大
// 初步思路是：
// CameraSystem 变成 CameraCollectorSystem
// 增加 Space::GetExecutedSystem，GraphicsSystem 获取 CameraCollectorSystem
// GraphicsSystem 遍历所有 Camera, 每个 Camera 对应一个 Context
// 使用稳定的 EntityID，并允许从 Component 获取 EntityID（不是必要的）
// 给 CameraComponent 增加一个字段，用于存储相机产物（没有产物 / 一个贴图）

pub struct HCameraSystem {
    pub cameras: Vec<(RcObject<HCamera>, nalgebra_glm::Mat4x4)>,
}

impl HCameraSystem {
    pub fn new() -> Self {
        Self { cameras: vec![] }
    }
}

impl super::traits::TSystem for HCameraSystem {
    fn tick_entity(&mut self, context: FSystemTickContext) {
        // 这段代码太麻烦了，试试看用宏简化？

        let transform: RcObject<HTransformComponent> =
            context.components[0].clone().try_downcast().unwrap();
        let transform_ref = transform.borrow();

        let camera: RcObject<HCameraComponent> =
            context.components[1].clone().try_downcast().unwrap();
        let camera_ref = camera.borrow_mut();
        if camera_ref.main_camera {
            camera_ref.camera.borrow_mut().target = hoo_engine()
                .borrow()
                .get_renderer()
                .get_main_viewport_target();
        }
        // let camera_desc = camera_ref.camera.borrow();

        self.cameras.push((
            camera_ref.camera.clone(),
            transform_ref.get_matrix_ignoring_scale(),
        ));
    }

    fn begin_frame(&mut self, _: &HSpace) {
        self.cameras.clear();
    }

    fn end_frame(&mut self, _: &HSpace) {}

    fn get_interested_components(&self) -> &'static [&'static [u32]] {
        &[&[COMPONENT_ID_TRANSFORM, COMPONENT_ID_CAMERA]]
    }
}
