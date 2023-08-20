#[derive(Clone)]
pub struct FColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl FColor {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

#[derive(Copy, Clone)]
pub enum ELightType {
    Directional,
    Point,
    Spot,
}

impl Into<u32> for ELightType {
    fn into(self) -> u32 {
        match self {
            ELightType::Directional => 0,
            ELightType::Point => 1,
            ELightType::Spot => 2,
        }
    }
}

#[derive(Clone)]
pub struct HLight {
    color: FColor,
    radius: f32,
    light_type: ELightType,
}

impl HLight {
    pub fn new(color: FColor, radius: f32, light_type: ELightType) -> Self {
        Self {
            color,
            radius,
            light_type,
        }
    }

    pub fn new_point(color: FColor, radius: f32) -> Self {
        Self::new(color, radius, ELightType::Point)
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct FShaderLight {
    pub position: nalgebra_glm::Vec3,
    _padding_0: u32,
    pub color: nalgebra_glm::Vec3,
    pub radius: f32,
    pub direction: nalgebra_glm::Vec3,
    pub light_type: u32,
}

impl FShaderLight {
    pub fn new_from_component(
        light: &HLight,
        position: &nalgebra_glm::Vec3,
        rotation: &nalgebra_glm::Quat,
    ) -> Self {
        let direction =
            nalgebra_glm::quat_rotate_vec3(rotation, &nalgebra_glm::vec3(0.0, -1.0, 0.0));
        Self {
            position: *position,
            _padding_0: 0,
            color: nalgebra_glm::Vec3::new(light.color.r, light.color.g, light.color.b),
            radius: light.radius,
            direction,
            light_type: light.light_type.into(),
        }
    }
}
