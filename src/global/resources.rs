use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

use crate::{
    renderer::{encoder::FDeviceEncoder, resource::TGPUResource},
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

    pub fn prepare_gpu_resources(
        &self,
        encoder: &mut FDeviceEncoder,
    ) -> Vec<RcMut<dyn TGPUResource>> {
        let strong_res: Vec<_> = {
            let gpu_resources_ref = self.gpu_resources.borrow_mut();
            gpu_resources_ref
                .iter()
                .filter_map(|weakref| weakref.upgrade())
                .collect()
        };

        let new_vec = strong_res
            .iter()
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

        let mut counter = 0u64;
        for res in strong_res.clone() {
            res.borrow_mut().assign_consolidation_id(counter);
            counter += 1;
        }

        strong_res
    }
}

struct XT<'a> {
    r: &'a i32,
}

fn foo2<'a>(x: &mut XT<'a>, y: &'a std::vec::Vec<std::cell::Ref<'a, i32>>) {
    x.r = &y[0];
}
