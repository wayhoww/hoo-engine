use nalgebra_glm as glm;

struct HTransformComponent {
    position: glm::Vec3,
    rotation: glm::Quat,
    scale: glm::Vec3,
}
