use crevice::std140::AsStd140;
use nalgebra::{Isometry3, Point3, Point4, Rotation3, Translation3, UnitQuaternion};

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Point4<f32>>,
    pub indexes: Vec<Point4<u32>>,
    pub rotation: UnitQuaternion<f32>,
    pub position: Point3<f32>,
}

impl Model {
    pub fn new(vertices: Vec<Point4<f32>>, indexes: Vec<Point4<u32>>) -> Self {
        Model {
            vertices,
            indexes,
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn with_isometry(
        vertices: Vec<Point4<f32>>,
        indexes: Vec<Point4<u32>>,
        rotation: UnitQuaternion<f32>,
        position: Point3<f32>,
    ) -> Self {
        Model { vertices, indexes, rotation, position }
    }
    pub fn get_uniform_info(&self) -> ModelUniformInfo {
        ModelUniformInfo {
            isometry: Isometry3::from_parts(
                Translation3::new(self.position.x, self.position.y, self.position.z),
                self.rotation,
            )
            .to_matrix()
            .into(),
            indexes_length: self.indexes.len() as u32,
        }
    }
}

#[derive(AsStd140)]
pub struct ModelUniformInfo {
    pub isometry: mint::ColumnMatrix4<f32>,
    pub indexes_length: u32,
}

impl ModelUniformInfo {
    pub fn as_std140(&self) -> <Self as AsStd140>::Std140Type {
        AsStd140::as_std140(self)
    }
}
