use hoo_object::RcObject;
use crate::object::*;

use super::TComponent;

pub struct HStaticModelComponent {
    mesh: RcObject<objects::HStaticMesh>
}

impl TComponent for HStaticModelComponent {
    fn component_name(&self) -> &'static str {
        return "HooEngine.HStaticModelComponent";   
    }

}