use crate::object::*;
use hoo_object::RcObject;
use nalgebra_glm as glm;

pub const COMPONENT_ID_STATIC_MESH: u32 = 0;
pub const COMPONENT_ID_TRANSFORM: u32 = 1;
pub const COMPONENT_ID_CAMERA: u32 = 2;
pub const COMPONENT_ID_LIGHT: u32 = 3;

pub struct HStaticModelComponent {
    pub model: RcObject<objects::HStaticModel>,
}

pub struct HTransformComponent {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}

impl HTransformComponent {
    pub fn new_face_at(position: &glm::Vec3, facing_point: &glm::Vec3, up: &glm::Vec3) -> Self {
        let direction = glm::normalize(&(position - facing_point)); // 新坐标轴的 y, -y 才是正方向
        let right = glm::normalize(&glm::cross(&direction, &up)); // 新坐标轴的 x
        let up = glm::cross(&right, &direction); // 新坐标轴的 z

        // let mut rotation_mat: glm::Mat3 = glm::zero();
        // rotation_mat.set_column(0, &right);
        // rotation_mat.set_column(1, &direction);
        // rotation_mat.set_column(2, &up);

        let rotation = nalgebra::UnitQuaternion::from_rotation_matrix(
            &nalgebra::Rotation3::from_basis_unchecked(&[right, direction, up]),
        );

        HTransformComponent {
            position: *position,
            rotation: *rotation.quaternion(),
            scale: glm::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn get_matrix(&self) -> glm::Mat4 {
        let mut matrix = glm::identity();
        matrix = glm::scaling(&self.scale) * matrix;
        matrix = glm::quat_to_mat4(&self.rotation) * matrix;
        matrix = glm::translation(&self.position) * matrix;
        matrix
    }

    pub fn get_matrix_ignoring_scale(&self) -> glm::Mat4 {
        let mut matrix = glm::identity();
        matrix = glm::quat_to_mat4(&self.rotation) * matrix;
        matrix = glm::translation(&self.position) * matrix;
        matrix
    }
}

pub struct HCameraComponent {
    pub camera: RcObject<objects::HCamera>,
}

pub struct HLightComponent {}
