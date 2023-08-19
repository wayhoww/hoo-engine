use hoo_object::{RcAny};

use crate::object::{space::HSpace};

pub trait TSystem {
    // 也是不太好的抽象。体现不了调用一次后不应该更改的特点。
    fn get_interest_components(&self) -> &'static [u32];

    fn begin_frame(&mut self, _space: &HSpace) {}
    fn end_frame(&mut self, _space: &HSpace) {}
    fn tick_entity(&mut self, space: &HSpace, delta_time: f64, components: Vec<RcAny>);
}
