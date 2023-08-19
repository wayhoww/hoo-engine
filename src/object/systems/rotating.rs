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

pub struct HRotatingSystem {
    counter: i64,
}

impl HRotatingSystem {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl super::traits::TSystem for HRotatingSystem {
    fn begin_frame(&mut self, space: &HSpace) {
        self.counter += 1;
    }

    fn tick_entity(
        &mut self,
        space: &HSpace,
        delta_time: f64,
        components: Vec<hoo_object::RcTrait<dyn Any>>,
    ) {
        let transform: RcObject<HTransformComponent> =
            components[0].clone().try_downcast().unwrap();

        let mut transform_ref = transform.borrow_mut();

        // let angle = self.counter as f32 * 0.01;
        // transform_ref.rotation = nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
    }

    fn get_interest_components(&self) -> &'static [u32] {
        return &[COMPONENT_ID_TRANSFORM];
    }
}
