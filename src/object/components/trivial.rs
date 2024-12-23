use std::{cell::RefCell, ops::Deref};

use crate::{
    object::{
        objects::{HMaterial, HStaticMesh, HStaticModel},
        *,
    },
    utils::RcMut,
};
use hoo_object::RcObject;
use lazy_static::lazy_static;
use nalgebra_glm as glm;

pub const COMPONENT_ID_STATIC_MODEL: u32 = 0;
pub const COMPONENT_ID_TRANSFORM: u32 = 1;
pub const COMPONENT_ID_CAMERA: u32 = 2;
pub const COMPONENT_ID_LIGHT: u32 = 3;
pub const COMPONENT_ID_AXIS: u32 = 4;

pub struct HStaticModelComponent {
    pub model: RcObject<objects::HStaticModel>,
}

pub struct HTransformComponent {
    pub position: glm::Vec3,
    pub rotation: glm::Quat,
    pub scale: glm::Vec3,
}

impl HTransformComponent {
    pub fn new_trs(position: &glm::Vec3, rotation: &glm::Quat, scale: &glm::Vec3) -> Self {
        HTransformComponent {
            position: *position,
            rotation: *rotation,
            scale: *scale,
        }
    }

    pub fn new_face_at(position: &glm::Vec3, facing_point: &glm::Vec3, up: &glm::Vec3) -> Self {
        let new_y = glm::normalize(&(position - facing_point)); // 新坐标轴的 y, -y 才是正方向
        let new_x = glm::normalize(&glm::cross(&new_y, up)); // 新坐标轴的 x
        let new_z = glm::cross(&new_x, &new_y); // 新坐标轴的 z

        let rotation = nalgebra::UnitQuaternion::from_rotation_matrix(
            &nalgebra::Rotation3::from_basis_unchecked(&[new_x, new_y, new_z]),
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
    pub main_camera: bool,
}

pub struct HLightComponent {
    pub light: RcObject<objects::HLight>,
}

pub struct HAxisComponent {} // empty

impl HAxisComponent {
    pub fn GetAxisModel() -> RcObject<HStaticModel> {
        thread_local! {
            static MODEL: RefCell<Option<RcObject<HStaticModel>>> = RefCell::new(None);
        }

        let out = MODEL.with(|static_model| {
            let mut static_model = static_model.borrow_mut();
            if let Some(static_model) = static_model.deref() {
                return static_model.clone();
            } else {
                let mesh = HStaticMesh::new("meshes/arrow.gltf");
                let mut material = HMaterial::new("shaders/main.wgsl");
                material.material.enable_shader_profile("model_axis".into());
                let model = RcObject::new(HStaticModel {
                    mesh: RcObject::new(mesh),
                    material: RcObject::new(material),
                });
                static_model.replace(model.clone());
                return model;
            }
        });

        out
    }
}
