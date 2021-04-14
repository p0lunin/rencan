/*#[derive(Debug, Clone)]
pub enum Intersection {
    Intersect {
        model_id: u32,
        triangle_idx: u32,
        vertices_offset: u32,
        barycentric_coords: [f32; 2],
        distance: f32,
    },
    NotIntersect,
}

impl Intersection {
    pub fn into_uniform(self) -> IntersectionUniform {
        match self {
            Intersection::NotIntersect => IntersectionUniform {
                intersect: 0,
                model_id: 0,
                triangle_idx: 0,
                vertices_offset: 0,
                barycentric_coords: mint::Vector2::from([0.0, 0.0]),
                distance: 0.0,
                paddings: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            },
            Intersection::Intersect {
                model_id,
                triangle_idx,
                barycentric_coords,
                vertices_offset,
                distance,
            } => IntersectionUniform {
                intersect: 1,
                model_id,
                triangle_idx,
                vertices_offset,
                barycentric_coords: mint::Vector2::from(barycentric_coords),
                distance,
                paddings: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            },
        }
    }
}*/

#[repr(C, packed)]
pub struct IntersectionUniform([f32; 34]);
