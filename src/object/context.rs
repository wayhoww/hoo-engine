use hoo_object::{into_trait, RcObject, RcTrait};

use super::{
    components::{
        HCameraComponent, HStaticModelComponent, HTransformComponent, COMPONENT_ID_CAMERA,
        COMPONENT_ID_STATIC_MESH, COMPONENT_ID_TRANSFORM,
    },
    entity::HEntity,
    objects::{HCamera, HMaterial, HStaticMesh, HStaticModel},
    space::HSpace,
    systems::{HCameraSystem, HGraphicsSystem, HRotatingSystem},
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
                material: RcObject::new(material),
            }),
        };
        entity1.add_component(COMPONENT_ID_STATIC_MESH, RcObject::new(model).into_any());
        space.entities.push(entity1);

        let entity2 = {
            let mut entity = HEntity::new();
            let transform_component = HTransformComponent::new_face_at(
                &glm::vec3(3.0, 3.0, 3.0),
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 0.0, 1.0),
            );
            entity.add_component(
                COMPONENT_ID_TRANSFORM,
                RcObject::new(transform_component).into_any(),
            );
            let camera_component = HCameraComponent {
                camera: RcObject::new(HCamera::new(
                    super::objects::FCameraProjection::Perspective {
                        fov: 45.0f32.to_radians(),
                        aspect: 800.0 / 600.0,
                        near: 0.1,
                        far: 1000.0,
                    },
                )),
            };
            entity.add_component(
                COMPONENT_ID_CAMERA,
                RcObject::new(camera_component).into_any(),
            );
            entity
        };
        space.entities.push(entity2);

        let graphics_system = RcObject::new(HGraphicsSystem::new());
        space.systems.push(into_trait!(graphics_system));

        let rotating_system = RcObject::new(HRotatingSystem::new());
        space.systems.push(into_trait!(rotating_system));

        let camera_system = RcObject::new(HCameraSystem::new());
        space.systems.push(into_trait!(camera_system));

        self.spaces.push(RcObject::new(space));
    }

    pub fn tick(&mut self, delta_time: f64) {
        for space in self.spaces.iter_mut() {
            space.borrow_mut().tick(delta_time);
        }
    }
}
