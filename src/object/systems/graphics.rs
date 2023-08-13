pub struct HGraphicsSystem {}

impl super::traits::TSystem for HGraphicsSystem {
    fn tick_entity(&mut self, delta_time: f64, components: Vec<hoo_object::RcTrait<dyn crate::object::components::TComponent>>) {

    }

    fn get_interest_components(&self) -> &'static [&'static str] {
        return &[ "HooEngine.HStaticModelComponent", "HooEngine.HTransformComponent" ];
    }
    
}
