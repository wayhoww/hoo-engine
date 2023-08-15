use std::any::Any;

use hoo_object::RcObject;

use crate::{
    device::{
        graphics::{FMaterial, FModel, FRenderObject},
        io::{load_binary, load_string},
    },
    hoo_engine,
    object::{components::*, objects::{HMaterial, HStaticModel}},
    rcmut,
};

pub struct HGraphicsSystem {}

impl HGraphicsSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::traits::TSystem for HGraphicsSystem {
    fn tick_entity(&mut self, delta_time: f64, components: Vec<hoo_object::RcTrait<dyn Any>>) {
        let static_model: RcObject<HStaticModelComponent> =
            components[0].clone().try_downcast().unwrap();
        let transform: RcObject<HTransformComponent> =
            components[1].clone().try_downcast().unwrap();

        let transform_ref = transform.borrow();

        // TODO: model 和 material 为啥要 view 呢？
        let model = static_model.borrow().model.borrow().assemble_model();
        let render_object = FRenderObject::new(model);

        hoo_engine()
            .borrow()
            .get_renderer_mut()
            .get_graphics_context()
            .add_render_object(render_object);
    }

    fn get_interest_components(&self) -> &'static [u32] {
        return &[COMPONENT_ID_STATIC_MESH, COMPONENT_ID_TRANSFORM];
    }
}
