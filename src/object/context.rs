use hoo_object::RcObject;

use super::space::HSpace;

pub struct HContext {
    spaces: Vec<RcObject<HSpace>>
}

impl HContext {
    pub fn new() -> Self {
        HContext {
            spaces: Vec::new(),
        }
    }

    pub fn register_component_types<T>(&mut self) {
        // TODO
    }
}