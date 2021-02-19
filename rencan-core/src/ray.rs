use nalgebra::Point3;

#[repr(C, align(32))]
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Point3<f32>,
}
