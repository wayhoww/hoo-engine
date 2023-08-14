use nalgebra_glm as glm;

pub struct HTransformComponent {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}
