use nalgebra::Point3;
use crevice::std140::AsStd140;

#[derive(Debug, Clone)]
pub struct HitBoxRectangle {
    min_point: Point3<f32>,
    max_point: Point3<f32>,
}

impl HitBoxRectangle {
    pub fn new() -> Self {
        HitBoxRectangle {
            min_point: Point3::new(std::f32::MAX, std::f32::MAX, std::f32::MAX),
            max_point: Point3::new(std::f32::MIN, std::f32::MIN, std::f32::MIN),
        }
    }

    pub fn update_by_point(&mut self, point: &Point3<f32>) {
        if self.min_point.x > point.x { self.min_point.x = point.x }
        if self.min_point.y > point.y { self.min_point.y = point.y }
        if self.min_point.z > point.z { self.min_point.z = point.z }

        if self.max_point.x < point.x { self.max_point.x = point.x }
        if self.max_point.y < point.y { self.max_point.y = point.y }
        if self.max_point.z < point.z { self.max_point.z = point.z }
    }

    pub fn into_uniform(self) -> HitBoxRectangleUniform {
        HitBoxRectangleUniform {
            min_point: self.min_point.coords.into(),
            max_point: self.max_point.coords.into(),
            align1: 0.0,
            align2: 0.0,
        }
    }
}

// I don't understand why it is not works without aligments
#[derive(AsStd140)]
pub struct HitBoxRectangleUniform {
    min_point: mint::Vector3<f32>,
    align1: f32,
    max_point: mint::Vector3<f32>,
    align2: f32,
}

pub type HitBoxRectangleUniformStd140 = <HitBoxRectangleUniform as AsStd140>::Std140Type;
