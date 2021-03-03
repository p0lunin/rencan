use crate::light::info::LightInfo;
use nalgebra::{Point3, UnitQuaternion};

#[derive(Debug, Clone)]
pub struct PointLight {
    pub info: LightInfo,
    pub position: Point3<f32>,
}

impl PointLight {
    pub fn new(info: LightInfo, position: Point3<f32>) -> Self {
        PointLight { info, position }
    }
    pub fn into_uniform(self) -> PointLightUniform {
        PointLightUniform {
            color: self.info.color.coords.into(),
            intensity: self.info.intensity,
            position: self.position.coords.into(),
        }
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct PointLightUniform {
    color: mint::Vector4<f32>,
    position: mint::Vector3<f32>,
    intensity: f32,
}
