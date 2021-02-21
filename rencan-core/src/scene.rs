use crate::{light::DirectionLight, Model};

pub struct Scene {
    pub models: Vec<Model>,
    pub global_light: DirectionLight,
}

impl Scene {
    pub fn new(models: Vec<Model>, global_light: DirectionLight) -> Self {
        Scene { models, global_light }
    }
}
