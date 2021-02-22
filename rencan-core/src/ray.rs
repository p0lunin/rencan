use nalgebra::Point4;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point4<f32>,
    pub direction: Point4<f32>,
}
