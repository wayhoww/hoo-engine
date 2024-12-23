use std::any::Any;

use hoo_object::RcObject;

use crate::object::{components::*, space::HSpace};

use super::FSystemTickContext;

pub struct HRotatingSystem {
    counter: i64,
}

impl HRotatingSystem {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl super::traits::TSystem for HRotatingSystem {
    fn begin_frame(&mut self, _space: &HSpace) {
        self.counter += 1;
    }

    fn tick_entity(&mut self, context: FSystemTickContext) {
        let transform: RcObject<HTransformComponent> =
            context.components[0].clone().try_downcast().unwrap();

        let mut transform_ref = transform.borrow_mut();

        let angle = self.counter as f32 * 0.001;
        transform_ref.rotation =
            nalgebra_glm::quat_angle_axis(angle, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
    }

    fn get_interested_components(&self) -> &'static [&'static [u32]] {
        &[&[COMPONENT_ID_TRANSFORM, COMPONENT_ID_STATIC_MODEL]]
    }
}
