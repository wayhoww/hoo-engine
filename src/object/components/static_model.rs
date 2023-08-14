use crate::object::*;
use hoo_object::RcObject;

pub struct HStaticModelComponent {
    pub mesh: RcObject<objects::HStaticMesh>,
}
