struct Ray {
    vec3 origin;
    vec4 direction;
    float max_distance;
};

struct ModelInfo {
    mat4 isometry;
    uint model_id;
    uint vertices_length;
    uint indexes_length;
    uint material;
    float albedo;
    float diffuse;
    float specular;
};

struct Intersection {
    vec3 point;
    vec3 normal;
    vec2 barycentric_coords;
    uint is_intersect;
    ModelInfo model;
    uint triangle_idx;
    uint vertices_offset;
    float distance;
    Ray ray;
    uint pixel_id;
};

Intersection intersection_succ(
    vec3 point,
    vec3 normal,
    ModelInfo model,
    uint triangle_idx,
    uint vertices_offset,
    vec2 barycentric_coords,
    float distance,
    Ray ray,
    uint pixel_id
) {
    return Intersection(point, normal, barycentric_coords, 1, model, triangle_idx, vertices_offset, distance, ray, pixel_id);
}

Intersection intersection_none() {
    Intersection intersect;
    intersect.is_intersect = 0;
    return intersect;
}

struct DirectLight {
    vec3 color;
    vec3 direction;
    float intensity;
};

struct Sphere {
    vec3 center;
    float radius;
};

struct HitBoxRectangle {
    vec3 min;
    vec3 max;
};

struct PointLight {
    vec3 color;
    vec3 position;
    float intensity;
};

const uint MATERIAL_DIFFUSE = 1;
const uint MATERIAL_MIRROR = 2;
