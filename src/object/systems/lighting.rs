use std::{any::Any, ops::Deref};

use hoo_object::RcObject;

use crate::object::{components::*, objects::FShaderLight, space::HSpace};

use super::FSystemTickContext;

pub struct HLightingSystem {
    lights: Vec<FShaderLight>,
}

impl HLightingSystem {
    pub fn new() -> Self {
        Self { lights: vec![] }
    }
}

impl HLightingSystem {
    pub fn get_lights(&self) -> &Vec<FShaderLight> {
        &self.lights
    }
}

impl super::traits::TSystem for HLightingSystem {
    fn begin_frame(&mut self, _space: &HSpace) {
        self.lights.clear();

        // end-frame 后，其他 system 会从中读取数据
    }

    fn tick_entity(&mut self, context: FSystemTickContext) {
        let transform: RcObject<HTransformComponent> =
            context.components[0].clone().try_downcast().unwrap();
        let transform_ref = transform.borrow();

        let light: RcObject<HLightComponent> =
            context.components[1].clone().try_downcast().unwrap();
        let light_ref = light.borrow();
        let light_desc = light_ref.light.borrow();

        let shader_light = FShaderLight::new_from_component(
            light_desc.deref(),
            &transform_ref.position,
            &transform_ref.rotation,
        );
        self.lights.push(shader_light);
    }

    fn get_interested_components(&self) -> &'static [&'static [u32]] {
        &[&[COMPONENT_ID_TRANSFORM, COMPONENT_ID_LIGHT]]
    }
}
