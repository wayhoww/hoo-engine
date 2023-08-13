use nalgebra_glm as glm;

use super::TComponent;

struct HTransformComponent {
    position: glm::Vec3,
    rotation: glm::Quat,
    scale: glm::Vec3,
}

impl TComponent for HTransformComponent {
    fn component_name(&self) -> &'static str {
        return "HooEngine.HTransformComponent";   
    }
}