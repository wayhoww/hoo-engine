use hoo_object::RcAny;

use crate::object::space::HSpace;

pub struct FSystemTickContext<'stack> {
    pub space: &'stack HSpace,
    pub delta_time: f64,
    pub group: usize,
    pub components: Vec<RcAny>,
    pub entity_id: u32,
}

pub trait TSystem {
    // 也是不太好的抽象。体现不了调用一次后不应该更改的特点。
    fn get_interested_components(&self) -> &'static [&'static [u32]];

    fn begin_frame(&mut self, _space: &HSpace) {}
    fn before_first_tick(&mut self, _space: &HSpace, _delta_time: f64) {}
    fn tick_entity(&mut self, context: FSystemTickContext);
    fn end_frame(&mut self, _space: &HSpace) {}
}
