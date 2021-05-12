use crate::hitbox::HitBoxRectangle;
use nalgebra::{Isometry3, Point3, Point4, Translation3, UnitQuaternion};

#[derive(Debug, Clone)]
pub enum Material {
    Phong { albedo: f32, diffuse: f32, specular: f32, color: [f32; 3] },
    Mirror,
}

impl Material {
    fn into_uniform(self) -> (f32, f32, f32, u32, [f32; 3]) {
        match self {
            Material::Phong { albedo, diffuse, specular, color } => (albedo, diffuse, specular, 1, color),
            Material::Mirror => (0.0, 0.0, 0.0, 2, [0.0; 3]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Point4<f32>>,
    pub indexes: Vec<Point4<u32>>,
    pub rotation: UnitQuaternion<f32>,
    pub position: Point3<f32>,
    pub scaling: f32,
    pub material: Material,
}

impl Model {
    pub fn new(vertices: Vec<Point4<f32>>, indexes: Vec<Point4<u32>>) -> Self {
        Model {
            vertices,
            indexes,
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
            scaling: 1.0,
            material: Material::Phong { albedo: 0.18, diffuse: 0.8, specular: 0.2, color: [1.0; 3] },
        }
    }
    pub fn with_isometry(
        vertices: Vec<Point4<f32>>,
        indexes: Vec<Point4<u32>>,
        rotation: UnitQuaternion<f32>,
        position: Point3<f32>,
        scaling: f32,
        material: Material,
    ) -> Self {
        Model { vertices, indexes, rotation, position, scaling, material }
    }
    pub fn get_uniform_info(&self, model_id: u32) -> ModelUniformInfo {
        let (albedo, diffuse, specular, material, color) = self.material.clone().into_uniform();
        let isometry = Isometry3::from_parts(
                Translation3::new(self.position.x, self.position.y, self.position.z),
                self.rotation,
            )
            .to_matrix() * self.scaling;
        let inverse_isometry = isometry.try_inverse().unwrap();
        ModelUniformInfo {
            isometry: isometry.into(),
            inverse_isometry: inverse_isometry.into(),
            model_id,
            _offset: 0,
            _offset2: 0,
            vertices_length: self.vertices.len() as u32,
            indexes_length: self.indexes.len() as u32,
            albedo,
            diffuse,
            specular,
            material,
            color
        }
    }
}

#[repr(C, packed)]
pub struct ModelUniformInfo {
    pub isometry: mint::ColumnMatrix4<f32>,
    pub inverse_isometry: mint::ColumnMatrix4<f32>,
    pub model_id: u32,
    pub vertices_length: u32,
    pub indexes_length: u32,
    pub _offset: u32,
    pub material: u32,
    pub albedo: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub color: [f32; 3],
    pub _offset2: u32
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

#[derive(Clone)]
pub struct SphereModel {
    pub center: Point3<f32>,
    pub radius: f32,
    pub rotation: UnitQuaternion<f32>,
    pub position: Point3<f32>,
    pub scaling: f32,
    pub material: Material,
}

impl SphereModel {
    pub fn new(center: Point3<f32>, radius: f32) -> Self {
        SphereModel {
            center,
            radius,
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
            scaling: 1.0,
            material: Material::Phong { albedo: 0.18, diffuse: 0.8, specular: 0.2, color: [1.0; 3] },
        }
    }

    pub fn get_uniform_info(&self, model_id: u32) -> ModelUniformInfo {
        let (albedo, diffuse, specular, material, color) = self.material.clone().into_uniform();
        let isometry = Isometry3::from_parts(
                Translation3::new(self.position.x, self.position.y, self.position.z),
                self.rotation,
            )
            .to_matrix() * self.scaling;
        let inverse_isometry = isometry.try_inverse().unwrap();
        ModelUniformInfo {
            isometry: isometry.into(),
            inverse_isometry: inverse_isometry.into(),
            model_id,
            vertices_length: 0,
            indexes_length: 0,
            albedo,
            diffuse,
            specular,
            material,
            _offset: 0,
            _offset2: 0,
            color
        }
    }
}
