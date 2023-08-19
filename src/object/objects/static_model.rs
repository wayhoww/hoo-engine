use hoo_object::RcObject;

use crate::device::graphics::*;
use crate::device::io::*;
use crate::*;

pub struct HMaterial {
    pub material: FMaterial,
}

impl HMaterial {
    pub fn new(shader_path: &str) -> Self {
        let mut mat = FMaterial::new(load_string(shader_path).unwrap());
        mat.enable_shader_profile("base".into());
        Self { material: mat }
    }
}

pub struct HStaticMesh {
    pub mesh: FMesh,
}

impl HStaticMesh {
    pub fn new(path: &str) -> Self {
        // TODO: 应当做烘焙
        let file_resource =
            editor::importer::load_gltf_from_slice(load_string(path).unwrap()).unwrap();
        Self {
            mesh: FMesh::from_file_resource(&file_resource[0].sub_meshes[0]),
        }
    }
}

pub struct HStaticModel {
    pub material: RcObject<HMaterial>,
    pub mesh: RcObject<HStaticMesh>,
}

impl HStaticModel {
    pub fn assemble_model(&self) -> FModel {
        let mesh = self.mesh.borrow().mesh.clone();
        let material = self.material.borrow().material.clone();

        // TODO: 这两是 view,不应该有 rcmut
        FModel::new(rcmut!(mesh), rcmut!(material))
    }
}
