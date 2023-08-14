use hoo_object::{RcAny, RcTrait};

use crate::object::components::*;

pub trait TSystem {
    // 也是不太好的抽象。体现不了调用一次后不应该更改的特点。
    fn get_interest_components(&self) -> &'static [u32];

    fn tick_entity(&mut self, delta_time: f64, components: Vec<RcAny>);
}
