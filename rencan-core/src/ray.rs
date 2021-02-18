use nalgebra::Point3;

#[repr(C, align(32))]
pub struct Ray {
    origin: Point3<f32>,
    direction: Point3<f32>,
}
