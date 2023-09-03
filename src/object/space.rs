use std::{collections::HashMap, ops::Deref, sync::atomic::AtomicU32};

use hoo_object::{RcAny, RcObject, RcTrait};

use crate::object::components::COMPONENT_ID_CAMERA;

use super::{
    components::{HAxisComponent, HCameraComponent, COMPONENT_ID_AXIS},
    entity::HEntity,
    systems::{FSystemTickContext, TSystem}, objects::HCamera,
};

pub struct HSpace {
    entities: HashMap<u32, HEntity>,
    pub systems: Vec<RcTrait<dyn TSystem>>,
    pub executed_systems: Vec<RcTrait<dyn TSystem>>,

    pub main_camera: Option<RcObject<HCamera>>,
    pub selected_entity_id: Option<u32>,
}

impl HSpace {
    pub fn new() -> Self {
        HSpace {
            entities: HashMap::new(),
            systems: Vec::new(),
            executed_systems: Vec::new(),
            main_camera: None,
            selected_entity_id: None,
        }
    }

    pub fn set_main_camera_entity(&mut self, entity_id: u32) {
        if let Some(entity) = self.entities.get(&entity_id) {
            if let Some(camera_component) = entity.components.get(&COMPONENT_ID_CAMERA) {
                if let Ok(camera_component) = camera_component.clone().try_downcast::<HCameraComponent>() {
                    self.main_camera = Some(camera_component.borrow().camera.clone());
                    return;
                }
            }
        }
        assert!(false);
    }

    pub fn add_entity(&mut self, entity: HEntity) -> u32 {
        // TODO: reusable id
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug_assert_ne!(id, u32::MAX);
        self.entities.insert(id, entity);
        return id;
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
        let hovered_id = self.main_camera.as_ref().unwrap().borrow().context.borrow().pipeline.borrow().get_properties().hovered_object_id.clone();
        self.selected_entity_id = hovered_id.borrow().clone();

        for system in self.systems.iter() {
            system.borrow_mut().begin_frame(self);
        }

        for system in self.systems.iter() {
            system.borrow_mut().before_first_tick(self, delta_time);
            for (entity_id, entity) in self.entities.iter() {
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
                        let context = FSystemTickContext {
                            space: self,
                            delta_time: delta_time,
                            group: i,
                            components: components,
                            entity_id: *entity_id,
                        };
                        system.borrow_mut().tick_entity(context);
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
