use hoo_object::{RcObject, RcTrait};

use super::{entity::HEntity, systems::TSystem, components::TComponent};

pub struct HSpace {
    pub entities: Vec<HEntity>,
    pub systems: Vec<RcTrait<dyn TSystem>>
}

impl HSpace {
    pub fn new() -> Self {
        HSpace {
            entities: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn tick(&mut self, delta_time: f64) {
        for system in self.systems.iter() {
            for entity in self.entities.iter() {
                let mut components: Vec<RcTrait<dyn TComponent>> = Vec::new();
                for component in entity.components.iter() {
                    if system.borrow().get_interest_components().contains(&component.borrow().component_name()) {
                        // let component: RcTrait<dyn TComponent> = component.try_downcast();
                        // components.push(component.clone());
                    }
                }
                // 暗示了禁止一个组件在一个 Entity 中出现多次，但没做检查
                if components.len() == system.borrow().get_interest_components().len() {
                    system.borrow_mut().tick_entity(delta_time, components);
                }
            }
        }
    }
}