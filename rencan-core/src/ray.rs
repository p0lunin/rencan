use nalgebra::Point4;

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point4<f32>,
    pub direction: Point4<f32>,
    pub max_distance: f32,
}

#[repr(C)]
pub struct LightRay([u32; 160]);
