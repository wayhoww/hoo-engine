

use crate::check;
use crate::io::resource::{RMesh, RSubMesh};
use crate::utils::bin_string_to_vec;

extern crate nalgebra_glm as glm;

pub fn load_gltf_from_slice<S: AsRef<[u8]>>(slice: S) -> Result<Vec<RMesh>, String> {
    let (document, buffers, _) = gltf::import_slice(slice).unwrap();

    let mut out_meshes: Vec<RMesh> = Vec::new();

    for mesh in document.meshes() {
        let mut out_mesh = RMesh::default();
        out_mesh.name = mesh.name().unwrap_or_default().into();

        for primitive in mesh.primitives() {
            let mut out_submesh = RSubMesh::default();
            out_submesh.name = format!("{:}.{:}", out_mesh.name, primitive.index());

            let indices = primitive.indices().ok_or("No indices")?;
            let view = indices.view().ok_or("No view for indices")?;

            check!(indices.count() * indices.data_type().size() == view.length());

            let buffer = &buffers[view.buffer().index()].0;
            let data = &buffer[view.offset()..view.offset() + view.length()];

            let out_indices = match indices.data_type() {
                gltf::accessor::DataType::U16 => Ok(bin_string_to_vec::<u16>(data)
                    .into_iter()
                    .map(|x| x as u32)
                    .collect()),
                gltf::accessor::DataType::U32 => Ok(bin_string_to_vec::<u32>(data)),
                _ => Err(format!("Unsupported index type: {:?}", indices.data_type())),
            }?;

            out_submesh.indices = out_indices;

            for (semantic, accessor) in primitive.attributes() {
                let view = accessor.view().unwrap();
                let buffer = &buffers[view.buffer().index()].0;
                let data = &buffer[view.offset()..view.offset() + view.length()];

                check!(
                    accessor.count()
                        * accessor.data_type().size()
                        * accessor.dimensions().multiplicity()
                        == view.length()
                );

                match semantic {
                    gltf::Semantic::Positions => {
                        check!(accessor.dimensions().multiplicity() == 3);
                        check!(accessor.data_type() == gltf::accessor::DataType::F32);
                        check!(out_submesh.positions.is_empty());

                        out_submesh.positions = bin_string_to_vec::<f32>(data)
                            .chunks(accessor.dimensions().multiplicity())
                            .map(|x| glm::vec3(x[0], x[1], x[2]))
                            .collect();
                    }
                    gltf::Semantic::Normals => {
                        check!(accessor.dimensions().multiplicity() == 3);
                        check!(accessor.data_type() == gltf::accessor::DataType::F32);
                        // check!(out_submesh.normals.is_none());

                        out_submesh.normals = bin_string_to_vec::<f32>(data)
                            .chunks(accessor.dimensions().multiplicity())
                            .map(|x| glm::vec3(x[0], x[1], x[2]))
                            .collect();
                    }
                    gltf::Semantic::TexCoords(0) => {
                        check!(accessor.dimensions().multiplicity() == 2);
                        check!(accessor.data_type() == gltf::accessor::DataType::F32);
                        // check!(out_submesh.uv0.is_none());

                        out_submesh.uv0 = bin_string_to_vec::<f32>(data)
                            .chunks(accessor.dimensions().multiplicity())
                            .map(|x| glm::vec2(x[0], x[1]))
                            .collect();
                    }
                    _ => {
                        eprintln!("Unsupported semantic: {:?}", semantic);
                    }
                }
            }
            out_submesh.check()?;
            out_mesh.sub_meshes.push(out_submesh);
        }
        out_meshes.push(out_mesh);
    }
    Ok(out_meshes)
}
