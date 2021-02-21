use crate::light::LightInfo;
use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub struct DirectionLight {
    pub info: LightInfo,
    pub direction: Vector3<f32>,
}

impl DirectionLight {
    pub fn new(info: LightInfo, direction: Vector3<f32>) -> Self {
        DirectionLight { info, direction }
    }
    pub fn into_uniform(self) -> DirectionLightUniform {
        DirectionLightUniform {
            color: self.info.color.coords.into(),
            intensity: self.info.intensity,
            direction: self.direction.into(),
        }
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct DirectionLightUniform {
    color: mint::Vector4<f32>,
    direction: mint::Vector3<f32>,
    intensity: f32,
}
