use std::any::Any;

use hoo_object::RcObject;

use crate::{
    device::graphics::FRenderObject,
    graphics::FPipelineContext,
    hoo_engine,
    object::{components::*, objects::HCameraTarget, space::HSpace},
};

use super::{HCameraSystem, HLightingSystem};

pub struct HGraphicsSystem {
    pipelines: Vec<FPipelineContext>,
}

impl HGraphicsSystem {
    pub fn new() -> Self {
        Self { pipelines: vec![] }
    }
}

impl super::traits::TSystem for HGraphicsSystem {
    fn begin_frame(&mut self, _space: &HSpace) {
        self.pipelines.clear();
    }

    fn before_first_tick(&mut self, space: &HSpace, _delta_time: f64) {
        let camera_systems = space.get_executed_systems_by_type::<HCameraSystem>();

        if let Some(camera_system) = camera_systems.first() {
            for (camera, transform) in camera_system.borrow().cameras.iter() {
                let camera = camera.borrow();
                let projection_mat = {
                    let mut proj = camera.camera_projection.clone();
                    if camera.auto_aspect {
                        let target_size = match camera.target.clone() {
                            HCameraTarget::Texture(target) => target.borrow().size(),
                            HCameraTarget::Screen => {
                                hoo_engine().borrow().get_renderer().get_swapchain_size()
                            }
                        };
                        let aspect_ratio = 1.0 * target_size.0 as f32 / target_size.1 as f32;
                        proj.set_aspect_ratio(aspect_ratio);
                    }
                    proj.get_projection_matrix()
                };

                let mut pipeline = FPipelineContext::new();
                pipeline.camera_transform = *transform;
                pipeline.camera_projection = projection_mat;
                pipeline.set_render_target(camera.target.clone());
                self.pipelines.push(pipeline);
            }
        }
    }

    fn tick_entity(
        &mut self,
        _space: &HSpace,
        _delta_time: f64,
        _: usize,
        components: Vec<hoo_object::RcTrait<dyn Any>>,
    ) {
        let transform: RcObject<HTransformComponent> =
            components[0].clone().try_downcast().unwrap();
        let static_model: RcObject<HStaticModelComponent> =
            components[1].clone().try_downcast().unwrap();

        let transform_ref = transform.borrow();

        // todo: better api?
        let transform = transform_ref.get_matrix();

        // TODO: model 和 material 为啥要 view 呢？
        let model = static_model.borrow().model.borrow().assemble_model();

        for pipeline in self.pipelines.iter_mut() {
            let mut render_object = FRenderObject::new(model.clone());
            render_object.set_transform_model(transform);
            pipeline.add_render_object(render_object);
        }
    }

    fn end_frame(&mut self, space: &HSpace) {
        let lighting_system = space.get_executed_systems_by_type::<HLightingSystem>();

        for mut pipeline in self.pipelines.clone() {
            if let Some(lighting_system) = lighting_system.first() {
                let lighting_system = lighting_system.borrow();
                let lights = lighting_system.get_lights();
                pipeline.set_lights(lights.clone());

                hoo_engine()
                    .borrow()
                    .get_renderer()
                    .submit_pipeline(pipeline.clone());
            }
        }

        self.pipelines.clear();
    }

    fn get_interested_components(&self) -> &'static [&'static [u32]] {
        &[
            &[COMPONENT_ID_TRANSFORM, COMPONENT_ID_STATIC_MODEL],
            &[COMPONENT_ID_TRANSFORM, COMPONENT_ID_AXIS],
        ]
    }
}
