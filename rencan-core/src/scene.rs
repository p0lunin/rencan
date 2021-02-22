use crate::{light::DirectionLight};
use crate::model::AppModel;

pub struct Scene {
    pub models: Vec<AppModel>,
    pub global_light: DirectionLight,
}

impl Scene {
    pub fn new(models: Vec<AppModel>, global_light: DirectionLight) -> Self {
        Scene { models, global_light }
    }
}
