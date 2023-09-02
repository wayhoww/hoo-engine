use std::{collections::HashMap, sync::atomic::AtomicU64};

use hoo_object::RcAny;

pub struct HEntity {
    pub components: HashMap<u32, RcAny>, // 绝对不应该这么做，但是先这样～
}

impl HEntity {
    pub fn new() -> Self {
        thread_local! {
            static COUNTER: AtomicU64 = AtomicU64::new(0)
        }

        HEntity {
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, component_id: u32, component: RcAny) {
        self.components.insert(component_id, component);
    }
}
