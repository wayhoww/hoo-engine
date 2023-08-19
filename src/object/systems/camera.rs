use std::any::Any;

use hoo_object::RcObject;

use crate::{
    device::{
        graphics::{FMaterial, FModel, FRenderObject},
        io::{load_binary, load_string},
    },
    graphics::{FGraphicsContext, FPipelineContext},
    hoo_engine,
    object::{
        components::*,
        objects::{HMaterial, HStaticModel},
        space::HSpace,
    },
    rcmut,
};

use super::HGraphicsSystem;

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
                let projection_mat = camera_desc.get_projection_matrix();

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

    fn get_interest_components(&self) -> &'static [u32] {
        return &[COMPONENT_ID_TRANSFORM, COMPONENT_ID_CAMERA];
    }
}
