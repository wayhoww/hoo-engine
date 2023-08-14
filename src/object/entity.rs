use std::collections::HashMap;

use hoo_object::{RcAny, RcTrait};

pub struct HEntity {
    pub components: HashMap<u32, RcAny>, // 绝对不应该这么做，但是先这样～
}

impl HEntity {
    pub fn new() -> Self {
        HEntity {
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, component_id: u32, component: RcAny) {
        self.components.insert(component_id, component);
    }
}
