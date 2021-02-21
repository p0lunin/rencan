use nalgebra::Point4;

#[derive(Debug, Clone)]
pub struct LightInfo {
    /// RGBA in range [0; 1]
    pub color: Point4<f32>,
    pub intensity: f32,
}

impl LightInfo {
    pub fn new(color: Point4<f32>, intensity: f32) -> Self {
        LightInfo { color, intensity }
    }
}
