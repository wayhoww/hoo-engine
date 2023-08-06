use serde::{Deserialize, Serialize};

use crate::check;

extern crate nalgebra_glm as glm;

pub trait TFileResource {
    fn name(&self) -> &str;
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RSubMesh {
    pub name: String,

    pub positions: Vec<glm::Vec3>,
    pub normals: Vec<glm::Vec3>,
    pub uv0: Vec<glm::Vec2>,

    pub indices: Vec<u32>,
}

impl TFileResource for RSubMesh {
    fn name(&self) -> &str {
        &self.name
    }
}

impl RSubMesh {
    pub fn check(&self) -> Result<(), String> {
        // check!(
        //     derivable!(self.normals.is_some() => self.positions.len() == self.normals.as_ref().unwrap().len())
        // );
        // check!(
        //     derivable!(self.uv0.is_some() => self.positions.len() == self.uv0.as_ref().unwrap().len())
        // );
        check!(!self.positions.is_empty());
        check!(self.positions.len() == self.normals.len());
        check!(self.positions.len() == self.uv0.len());

        check!(self.indices.len() % 3 == 0);
        check!(!self.indices.is_empty());
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RMesh {
    pub name: String,

    pub sub_meshes: Vec<RSubMesh>,
}

impl TFileResource for RMesh {
    fn name(&self) -> &str {
        &self.name
    }
}
