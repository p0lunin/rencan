use nalgebra::Point4;

pub struct Model {
    pub vertices: Vec<Point4<f32>>,
    pub indexes: Vec<Point4<u32>>,
}

impl Model {
    pub fn new(vertices: Vec<Point4<f32>>, indexes: Vec<Point4<u32>>) -> Self {
        Model { vertices, indexes }
    }
}
