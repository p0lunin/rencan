struct Ray {
    vec3 origin;
    vec4 direction;
    float max_distance;
};

struct Intersection {
    vec3 point;
    vec3 normal;
    vec2 barycentric_coords;
    uint is_intersect;
    uint model_id;
    uint triangle_idx;
    uint vertices_offset;
    float distance;
};

Intersection intersection_succ(
    vec3 point,
    vec3 normal,
    uint model_id,
    uint triangle_idx,
    uint vertices_offset,
    vec2 barycentric_coords,
    float distance
) {
    return Intersection(point, normal, barycentric_coords, 1, model_id, triangle_idx, vertices_offset, distance);
}

Intersection intersection_none() {
    return Intersection(vec3(0.0), vec3(0.0), vec2(0.0), 0, 0, 0, 0, 0.0);
}

struct DirectLight {
    vec3 color;
    vec3 direction;
    float intensity;
};

struct ModelInfo {
    mat4 isometry;
    uint model_id;
    uint vertices_length;
    uint indexes_length;
    float albedo;
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
