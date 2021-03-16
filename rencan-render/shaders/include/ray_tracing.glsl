const float eps = 0.0001;

struct IntersectResult {
    vec3 normal;
    vec2 barycentric_coords;
    float distance;
    bool intersect;
};

IntersectResult not_intersect() {
    const IntersectResult empty = IntersectResult(vec3(0.0), vec2(0.0), 0.0, false);
    return empty;
}
IntersectResult ret_intersect(vec3 normal, vec2 coords, float t) {
    return IntersectResult(normal, coords, t, true);
}

IntersectResult _intersect(Ray ray, vec3[3] triangle) {
    vec3 v0v1 = triangle[1] - triangle[0];
    vec3 v0v2 = triangle[2] - triangle[0];
    vec3 pvec = cross(ray.direction.xyz, v0v2);
    float det = dot(v0v1, cross(ray.direction.xyz, v0v2));

    if (det < eps) return not_intersect();

    float inv_det = 1.0 / det;

    vec3 tvec = ray.origin - triangle[0];
    float u = dot(tvec, pvec) * inv_det;
    if (u < 0 || u > 1) return not_intersect();

    vec3 vvec = cross(tvec, v0v1);
    float v = dot(ray.direction.xyz, vvec) * inv_det;
    if (v < 0 || u + v > 1) return not_intersect();

    float t = dot(v0v2, vvec) * inv_det;
    if (t < 0) return not_intersect();

    vec3 normal = normalize(cross(v0v1, v0v2));

    return ret_intersect(normal, vec2(u, v), t);
}

vec3 _intersect_box(HitBoxRectangle hit_box, Ray ray) {
    vec3 rad = hit_box.max - hit_box.min;
    ray.origin = ray.origin - hit_box.min;

    vec3 m = 1.0/ray.direction.xyz;
    vec3 n = m*ray.origin;
    vec3 k = abs(m)*rad;
    vec3 t1 = -n - k;
    vec3 t2 = -n + k;

    float tN = max( max( t1.x, t1.y ), t1.z );
    float tF = min( min( t2.x, t2.y ), t2.z );

    if( tN>tF || tF<0.0) return vec3(0.0);

    return vec3(1.0, tN, tF);
}

Intersection trace(
    Ray origin_ray
) {
    Intersection inter = intersection_none();
    float distance = 1.0 / 0.0;

    Ray ray = origin_ray;

    uint offset_vertices = 0;
    uint offset_indexes = 0;

    for (int model_idx = 0; model_idx < model_counts; model_idx++) {
        HitBoxRectangle hit_box = hit_boxes[model_idx];
        ModelInfo model = models[model_idx];

        mat4 global_to_model = inverse(model.isometry);
        ray.origin = (global_to_model * vec4(origin_ray.origin, 1.0)).xyz;
        ray.direction = global_to_model * origin_ray.direction;

        vec3 is_inter_hitbox = _intersect_box(hit_box, ray);

        if (is_inter_hitbox.x == 0.0 || is_inter_hitbox.y > ray.max_distance) {
            offset_indexes += model.indexes_length;
            offset_vertices += model.vertices_length;
            continue;
        }

        for (int i = 0; i < model.indexes_length; i++) {
            uvec3 index = indexes[offset_indexes + i];
            vec3 vertice1 = vertices[offset_vertices + index.x];
            vec3 vertice2 = vertices[offset_vertices + index.y];
            vec3 vertice3 = vertices[offset_vertices + index.z];
            vec3[3] vertices = vec3[](vertice1, vertice2, vertice3);
            IntersectResult res = _intersect(ray, vertices);
            if (res.intersect && res.distance < distance && res.distance < ray.max_distance) {
                vec3 inter_point = origin_ray.origin + origin_ray.direction.xyz * res.distance;
                distance = res.distance;
                inter = intersection_succ(
                    inter_point,
                    res.normal,
                    model_idx,
                    offset_indexes + i,
                    offset_vertices,
                    res.barycentric_coords,
                    res.distance
                );
            }
        }
        offset_indexes += model.indexes_length;
        offset_vertices += model.vertices_length;
    }

    return inter;
}
