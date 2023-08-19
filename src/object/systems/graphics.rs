use std::any::Any;

use hoo_object::RcObject;

use crate::{
    device::{
        graphics::{FMaterial, FModel, FRenderObject},
        io::{load_binary, load_string},
    },
    hoo_engine,
    object::{components::*, objects::{HMaterial, HStaticModel}},
    rcmut, graphics::{FGraphicsContext, FPipelineContext},
};

pub struct HGraphicsSystem {
    graphics_context: Option<FPipelineContext>
}

impl HGraphicsSystem {
    pub fn new() -> Self {
        Self { graphics_context: None }
    }
}

impl super::traits::TSystem for HGraphicsSystem {
    fn begin_frame(&mut self) {
        let context = FPipelineContext::new();
        self.graphics_context = Some(context);
    }

    fn tick_entity(&mut self, delta_time: f64, components: Vec<hoo_object::RcTrait<dyn Any>>) {
        let static_model: RcObject<HStaticModelComponent> =
            components[0].clone().try_downcast().unwrap();
        let transform: RcObject<HTransformComponent> =
            components[1].clone().try_downcast().unwrap();

        let transform_ref = transform.borrow();

        // todo: better api?
        let transform = nalgebra_glm::identity();
        let transform = nalgebra_glm::scale(&transform, &transform_ref.scale);
        let transform = nalgebra_glm::quat_to_mat4(&transform_ref.rotation) * transform;
        let transform = nalgebra_glm::translate(&transform, &transform_ref.position);


        // TODO: model 和 material 为啥要 view 呢？
        let model = static_model.borrow().model.borrow().assemble_model();
        let mut render_object = FRenderObject::new(model);
        render_object.set_transform_model(transform);
        self.graphics_context.as_mut().unwrap().add_render_object(render_object);
    }

    fn end_frame(&mut self) {
        hoo_engine().borrow().get_renderer().submit_pipeline(self.graphics_context.take().unwrap());
    }

    fn get_interest_components(&self) -> &'static [u32] {
        return &[COMPONENT_ID_STATIC_MESH, COMPONENT_ID_TRANSFORM];
    }
}
