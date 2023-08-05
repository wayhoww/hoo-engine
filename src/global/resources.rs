use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    renderer::{encoder::FWebGPUEncoder, resource::TGPUResource},
    utils::types::RcMut,
};

pub struct FGlobalResources {
    gpu_resources: RefCell<Vec<Weak<RefCell<dyn TGPUResource>>>>,
}

impl FGlobalResources {
    pub fn new() -> Self {
        Self {
            gpu_resources: RefCell::new(Vec::new()),
        }
    }

    pub fn add_gpu_resource(&self, resource: RcMut<dyn TGPUResource>) {
        // 内部可变模式。
        // 不然锁的粒度太大了
        self.gpu_resources
            .borrow_mut()
            .push(Rc::downgrade(&resource));
    }

    pub fn prepare_gpu_resources(&self, encoder: &mut FWebGPUEncoder) {
        let new_vec = self
            .gpu_resources
            .borrow_mut()
            .iter()
            .filter_map(|weakref| weakref.upgrade())
            .map(|rc_res| {
                let mut resource = rc_res.borrow_mut();
                if !resource.ready() {
                    resource.create_device_resource(encoder);
                };
                if resource.need_update() {
                    resource.update_device_resource(encoder);
                };
                Rc::downgrade(&rc_res)
            })
            .collect::<Vec<_>>();

        self.gpu_resources.replace(new_vec);
    }
}
