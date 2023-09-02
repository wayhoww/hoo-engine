use hoo_object::{RcAny, RcObject, RcTrait};

use super::{entity::HEntity, systems::TSystem};

pub struct HSpace {
    pub entities: Vec<HEntity>,
    pub systems: Vec<RcTrait<dyn TSystem>>,
    pub executed_systems: Vec<RcTrait<dyn TSystem>>,
}

pub struct FSystemTickStruct<'stack> {
    pub space: &'stack HSpace,
    pub delta_time: f64,
    pub group: usize,
    pub components: Vec<RcAny>,
    pub entity_id: usize,
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
            system.borrow_mut().before_first_tick(self, delta_time);
            for entity in self.entities.iter() {
                let groups = system.borrow().get_interested_components();
                for (i, interested_component_group) in groups.iter().enumerate() {
                    let mut components: Vec<RcAny> = Vec::new();
                    for interested_component in interested_component_group.iter() {
                        if let Some(component) = entity.components.get(interested_component) {
                            components.push(component.clone());
                        } else {
                            break;
                        }
                    }
                    if components.len() == interested_component_group.len() {
                        system
                            .borrow_mut()
                            .tick_entity(self, delta_time, i, components);
                    }
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
