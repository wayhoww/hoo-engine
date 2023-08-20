use hoo_object::{RcAny, RcObject, RcTrait};

use super::{entity::HEntity, systems::TSystem};

pub struct HSpace {
    pub entities: Vec<HEntity>,
    pub systems: Vec<RcTrait<dyn TSystem>>,
    pub executed_systems: Vec<RcTrait<dyn TSystem>>,
}

impl HSpace {
    pub fn new() -> Self {
        HSpace {
            entities: Vec::new(),
            systems: Vec::new(),
            executed_systems: Vec::new(),
        }
    }

    pub fn get_systems_by_type<T: TSystem>(&self) -> Vec<RcObject<T>> {
        let mut result: Vec<RcObject<T>> = Vec::new();
        for system in self.systems.iter() {
            if let Ok(sys) = system.clone().try_downcast::<T>() {
                result.push(sys);
            }
        }
        result
    }

    pub fn get_executed_systems_by_type<T: TSystem>(&self) -> Vec<RcObject<T>> {
        let mut result: Vec<RcObject<T>> = Vec::new();
        for system in self.executed_systems.iter() {
            if let Ok(sys) = system.clone().try_downcast::<T>() {
                result.push(sys);
            }
        }
        result
    }

    pub fn tick(&mut self, delta_time: f64) {

        for system in self.systems.iter() {
            system.borrow_mut().begin_frame(self);
        }

        for system in self.systems.iter() {
            for entity in self.entities.iter() {
                let mut components: Vec<RcAny> = Vec::new();
                for interested_component in system.borrow().get_interested_components() {
                    if let Some(component) = entity.components.get(interested_component) {
                        components.push(component.clone());
                    } else {
                        break;
                    }
                }
                if components.len() == system.borrow().get_interested_components().len() {
                    system
                        .borrow_mut()
                        .tick_entity(self, delta_time, components);
                }
            }
            self.executed_systems.push(system.clone());
        }

        for system in self.systems.iter() {
            system.borrow_mut().end_frame(self);
        }

        self.executed_systems.clear();
    }
}
