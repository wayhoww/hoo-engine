use crate::{device::graphics::FTexture, hoo_engine, utils::RcMut};

#[derive(Clone)]
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

impl FCameraProjection {
    pub fn get_projection_matrix(&self) -> nalgebra_glm::Mat4 {
        match self {
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

    pub fn set_aspect_ratio(&mut self, ar: f32) {
        match self {
            FCameraProjection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => *aspect = ar,
            FCameraProjection::Orthographic {
                width,
                height,
                near,
                far,
            } => *width = *height * ar,
        }
    }
}

#[derive(Clone, Default)]
pub enum HCameraTarget {
    #[default]
    Screen,
    Texture(RcMut<FTexture>),
}

pub struct HCamera {
    pub camera_projection: FCameraProjection,
    pub auto_aspect: bool,
    pub target: HCameraTarget,
}

impl HCamera {
    pub fn new(camera_projection: FCameraProjection) -> Self {
        Self {
            camera_projection,
            auto_aspect: true,
            target: hoo_engine()
                .borrow()
                .get_renderer()
                .get_main_viewport_target(),
        }
    }

    pub fn set_target(&mut self, target: HCameraTarget) {
        self.target = target;
    }

    pub fn get_projection_matrix(&self) -> nalgebra_glm::Mat4 {
        return self.camera_projection.get_projection_matrix();
    }
}
