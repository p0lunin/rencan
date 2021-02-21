use crate::light::info::LightInfo;
use nalgebra::{Point3, UnitQuaternion};

#[derive(Debug, Clone)]
pub struct PointLight {
    pub info: LightInfo,
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

impl PointLight {
    pub fn new(info: LightInfo, position: Point3<f32>, rotation: UnitQuaternion<f32>) -> Self {
        PointLight { info, position, rotation }
    }
}
