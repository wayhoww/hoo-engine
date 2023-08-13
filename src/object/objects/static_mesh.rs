use hoo_meta;
use hoo_meta_macros::*;
use hoo_object::RcObject;

use crate::device::graphics;

#[derive(JsStructNoConstructor)]
struct StaticMesh {
    model: graphics::FMesh
}