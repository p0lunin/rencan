use crate::hitbox::HitBoxRectangle;
use crevice::std140::AsStd140;
use nalgebra::{Isometry3, Point3, Point4, Translation3, UnitQuaternion};

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Point4<f32>>,
    pub indexes: Vec<Point4<u32>>,
    pub rotation: UnitQuaternion<f32>,
    pub position: Point3<f32>,
    pub scaling: f32,
    pub albedo: f32,
    pub specularity: f32,
}

impl Model {
    pub fn new(vertices: Vec<Point4<f32>>, indexes: Vec<Point4<u32>>) -> Self {
        Model {
            vertices,
            indexes,
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
            scaling: 1.0,
            albedo: 0.18,
            specularity: 0.0,
        }
    }
    pub fn with_isometry(
        vertices: Vec<Point4<f32>>,
        indexes: Vec<Point4<u32>>,
        rotation: UnitQuaternion<f32>,
        position: Point3<f32>,
        scaling: f32,
        albedo: f32,
        specularity: f32,
    ) -> Self {
        Model { vertices, indexes, rotation, position, scaling, albedo, specularity }
    }
    pub fn get_uniform_info(&self, model_id: u32) -> ModelUniformInfo {
        ModelUniformInfo {
            isometry: (Isometry3::from_parts(
                Translation3::new(self.position.x, self.position.y, self.position.z),
                self.rotation,
            )
            .to_matrix()
                * self.scaling)
                .into(),
            model_id,
            vertices_length: self.vertices.len() as u32,
            indexes_length: self.indexes.len() as u32,
            albedo: self.albedo,
            specularity: self.specularity,
            offsets: mint::Vector2 {
                x: 0.0,
                y: 0.0,
            }
        }
    }
}

#[derive(AsStd140)]
pub struct ModelUniformInfo {
    pub isometry: mint::ColumnMatrix4<f32>,
    pub model_id: u32,
    pub vertices_length: u32,
    pub indexes_length: u32,
    pub albedo: f32,
    pub specularity: f32,
    pub offsets: mint::Vector2<f32>,
}

impl ModelUniformInfo {
    pub fn as_std140(&self) -> <Self as AsStd140>::Std140Type {
        AsStd140::as_std140(self)
    }
}

#[allow(dead_code)]
pub struct AppModel {
    model: Model,
    hit_box: HitBoxRectangle,
    need_update_info: bool,
    need_update_vertices: bool,
    need_update_indices: bool,
}

impl AppModel {
    pub fn hit_box(&self) -> &HitBoxRectangle {
        &self.hit_box
    }
}

impl AppModel {
    pub fn new(model: Model) -> Self {
        let mut hit_box = HitBoxRectangle::new();
        model.vertices.iter().for_each(|v| hit_box.update_by_point(&Point3::new(v.x, v.y, v.z)));
        Self {
            model,
            hit_box,
            need_update_info: true,
            need_update_vertices: true,
            need_update_indices: true,
        }
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn update_info(&mut self, f: impl FnOnce(&mut Model)) {
        f(&mut self.model);
        self.need_update_info = true;
    }
}
