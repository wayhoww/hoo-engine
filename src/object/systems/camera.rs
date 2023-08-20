use std::any::Any;

use hoo_object::RcObject;

use crate::{
    hoo_engine,
    object::{components::*, space::HSpace},
};

use super::HGraphicsSystem;

// 有实际需求的时候再考虑多相机的问题，修改不是很大
// 初步思路是：
// CameraSystem 变成 CameraCollectorSystem
// 增加 Space::GetExecutedSystem，GraphicsSystem 获取 CameraCollectorSystem
// GraphicsSystem 遍历所有 Camera, 每个 Camera 对应一个 Context
// 使用稳定的 EntityID，并允许从 Component 获取 EntityID（不是必要的）
// 给 CameraComponent 增加一个字段，用于存储相机产物（没有产物 / 一个贴图）

pub struct HCameraSystem {
    found: bool,
}

impl HCameraSystem {
    pub fn new() -> Self {
        Self { found: false }
    }
}

impl super::traits::TSystem for HCameraSystem {
    fn tick_entity(
        &mut self,
        space: &HSpace,
        _delta_time: f64,
        components: Vec<hoo_object::RcTrait<dyn Any>>,
    ) {
        // 这段代码太麻烦了，试试看用宏简化？

        let transform: RcObject<HTransformComponent> =
            components[0].clone().try_downcast().unwrap();
        let transform_ref = transform.borrow();

        let camera: RcObject<HCameraComponent> = components[1].clone().try_downcast().unwrap();
        let camera_ref = camera.borrow();
        let camera_desc = camera_ref.camera.borrow();

        if self.found {
            todo!("log error here");
        } else {
            self.found = true;
            // TODO: multiple viewport in a same space
            let graphics_systems = space.get_systems_by_type::<HGraphicsSystem>();

            for sys in graphics_systems {
                let transform_mat = transform_ref.get_matrix_ignoring_scale();
                let projection_mat = {
                    let mut proj = camera_desc.camera_projection.clone();
                    if camera_ref.auto_aspect {
                        let swapchain_size =
                            hoo_engine().borrow().get_renderer().get_swapchain_size();
                        let aspect_ratio = 1.0 * swapchain_size.0 as f32 / swapchain_size.1 as f32;
                        proj.set_aspect_ratio(aspect_ratio);
                    }
                    proj.get_projection_matrix()
                };

                let mut sys_ref = sys.borrow_mut();
                sys_ref
                    .get_context_mut()
                    .set_camera_transform(transform_mat);
                sys_ref
                    .get_context_mut()
                    .set_camera_projection(projection_mat);
            }
        }
    }

    fn begin_frame(&mut self, _: &HSpace) {
        self.found = false;
    }

    fn end_frame(&mut self, _: &HSpace) {
        if !self.found {
            todo!("log error here");
        }
    }

    fn get_interested_components(&self) -> &'static [u32] {
        &[COMPONENT_ID_TRANSFORM, COMPONENT_ID_CAMERA]
    }
}
