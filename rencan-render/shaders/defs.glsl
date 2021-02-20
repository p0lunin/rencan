struct Ray {
    vec3 origin;
    vec4 direction;
};

struct Intersection {
    vec2 barycentric_coords;
    uint is_intersect;
    uint model_id;
    uint triangle_idx;
    float distance;
};

Intersection intersection_succ(uint model_id,
    uint triangle_idx,
    vec2 barycentric_coords,
    float distance
) {
    return Intersection(barycentric_coords, 1, model_id, triangle_idx, distance);
}

Intersection intersection_none() {
    return Intersection(vec2(0.0), 0, 0, 0, 0.0);
}
