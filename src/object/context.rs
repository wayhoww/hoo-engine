use hoo_object::{into_trait, RcObject, RcTrait};

use super::{
    components::{
        HStaticModelComponent, HTransformComponent, COMPONENT_ID_STATIC_MESH,
        COMPONENT_ID_TRANSFORM,
    },
    entity::HEntity,
    objects::{HStaticMesh, HStaticModel, HMaterial},
    space::HSpace,
    systems::{HGraphicsSystem, HRotatingSystem},
};

use nalgebra_glm as glm;

pub struct HContext {
    spaces: Vec<RcObject<HSpace>>,
}

impl HContext {
    pub fn new() -> Self {
        HContext { spaces: Vec::new() }
    }

    pub fn create_demo_space(&mut self) {
        let mut space = HSpace::new();

        let mut entity1 = HEntity::new();
        let transform_component1 = HTransformComponent {
            position: glm::vec3(0.0, 0.0, 0.0),
            rotation: glm::quat(0.0, 0.0, 0.0, 1.0),
            scale: glm::vec3(1.0, 1.0, 1.0),
        };
        // TODO: 这个地方要做动态检查
        entity1.add_component(
            COMPONENT_ID_TRANSFORM,
            RcObject::new(transform_component1).into_any(),
        );

        let mesh = HStaticMesh::new("meshes/cube.gltf");
        let material = HMaterial::new("shaders/main.wgsl");
        let model = HStaticModelComponent {
            model: RcObject::new(HStaticModel {
                mesh: RcObject::new(mesh),
                material: RcObject::new(material)
            }),
        };
        entity1.add_component(COMPONENT_ID_STATIC_MESH, RcObject::new(model).into_any());
        space.entities.push(entity1);

        let graphics_system = RcObject::new(HGraphicsSystem::new());
        space.systems.push(into_trait!(graphics_system));

        let rotating_system = RcObject::new(HRotatingSystem::new());
        space.systems.push(into_trait!(rotating_system));

        self.spaces.push(RcObject::new(space));
    }

    pub fn tick(&mut self, delta_time: f64) {
        for space in self.spaces.iter_mut() {
            space.borrow_mut().tick(delta_time);
        }
    }
}
