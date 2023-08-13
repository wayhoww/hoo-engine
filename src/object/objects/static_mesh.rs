use hoo_meta;
use hoo_meta_macros::*;
use hoo_object::RcObject;

use crate::device::graphics;
use crate::*;

struct StaticMesh {
    mesh: graphics::FMesh,
}

impl StaticMesh {
    pub fn new(path: String) -> Self {
        // TODO: 应当做烘焙
        let file_resource = editor::importer::load_gltf_from_slice(bundle::gltf_cube()).unwrap();
        Self {
            mesh: graphics::FMesh::from_file_resource(&file_resource[0].sub_meshes[0]),
        }
    }
}
