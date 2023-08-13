use hoo_object::{RcAny, RcTrait};

use super::components::TComponent;

pub struct HEntity {
    pub components: Vec<RcTrait<dyn TComponent>>,   // 绝对不应该这么做，但是先这样～
}
