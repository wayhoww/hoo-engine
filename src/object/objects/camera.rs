pub enum FCameraProjection {
    Perspective {
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    #[allow(dead_code)]
    Orthographic {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Default for FCameraProjection {
    fn default() -> Self {
        FCameraProjection::Perspective {
            fov: 45.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

pub struct HCamera {
    pub camera_projection: FCameraProjection,
}

impl HCamera {
    pub fn new(camera_projection: FCameraProjection) -> Self {
        Self { camera_projection }
    }

    pub fn get_projection_matrix(&self) -> nalgebra_glm::Mat4 {
        match &self.camera_projection {
            FCameraProjection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => nalgebra_glm::perspective(*aspect, *fov, *near, *far),
            FCameraProjection::Orthographic {
                width,
                height,
                near,
                far,
            } => nalgebra_glm::ortho(
                -*width / 2.0,
                *width / 2.0,
                -*height / 2.0,
                *height / 2.0,
                *near,
                *far,
            ),
        }
    }
}
