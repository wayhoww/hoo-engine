use hoo_object::{RcAny, RcObject, RcTrait};

use super::{entity::HEntity, systems::TSystem};

pub struct HSpace {
    pub entities: Vec<HEntity>,
    pub systems: Vec<RcTrait<dyn TSystem>>,
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
                let mut components: Vec<RcAny> = Vec::new();
                for interested_component in system.borrow().get_interest_components() {
                    if let Some(component) = entity.components.get(interested_component) {
                        components.push(component.clone());
                    } else {
                        break;
                    }
                }
                if components.len() == system.borrow().get_interest_components().len() {
                    system.borrow_mut().tick_entity(delta_time, components);
                }
            }
        }
    }
}
