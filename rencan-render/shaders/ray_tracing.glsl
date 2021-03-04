#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly buffer Rays {
    Ray rays[];
};
layout(std140, set = 0, binding = 2) writeonly buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 1, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 1, binding = 1) readonly buffer ModelInfos {
    ModelInfo models[];
};
layout(set = 1, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 1, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

const float eps = 0.001;

bool eqf(float f1, float f2) {
    return abs(f1 - f2) < eps;
}

struct IntersectResult {
    vec2 barycentric_coords;
    float distance;
    bool intersect;
};

IntersectResult not_intersect() {
    return IntersectResult(vec2(0.0), 0.0, false);
}
IntersectResult ret_intersect(vec2 coords, float t) {
    return IntersectResult(coords, t, true);
}

IntersectResult intersect(Ray ray, vec3[3] triangle) {
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

    return ret_intersect(vec2(u, v), t);
}

vec4 check_intersect_hitbox(HitBoxRectangle hit_box, Ray ray) {
    vec3 tmin = (hit_box.min - ray.origin) / ray.direction.xyz;
    vec3 tmax = (hit_box.max - ray.origin) / ray.direction.xyz;

    if (tmin.x > tmax.x) {
        float temp = tmin.x;
        tmin.x = tmax.x;
        tmax.x = temp;
    }

    if (tmin.y > tmax.y) {
        float temp = tmin.y;
        tmin.y = tmax.y;
        tmax.y = temp;
    }

    if (tmin.z > tmax.z) {
        float temp = tmin.z;
        tmin.z = tmax.z;
        tmax.z = temp;
    }

    if ((tmin.x > tmax.y) || (tmin.y > tmax.x))
        return vec4(0.0);

    if (tmin.y > tmin.x) {
        tmin.x = tmin.y;
    }

    if (tmax.y < tmax.x) {
        tmax.x = tmax.y;
    }

    if ((tmin.x > tmax.z) || (tmin.z > tmax.x))
        return vec4(0.0);

    if (tmin.z > tmin.x)
        tmin.x = tmin.z;

    if (tmax.z < tmax.x)
        tmax.x = tmax.z;

    return vec4(1.0, tmin);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    Intersection inter = intersection_none();
    float distance = 1.0 / 0.0;

    Ray origin_ray = rays[idx];
    Ray ray = origin_ray;

    uint offset_vertices = 0;
    uint offset_indexes = 0;

    for (int model_idx = 0; model_idx < model_counts; model_idx++) {
        HitBoxRectangle hit_box = hit_boxes[model_idx];

        ModelInfo model = models[model_idx];

        mat4 global_to_model = inverse(model.isometry);
        ray.origin = (global_to_model * vec4(origin_ray.origin, 1.0)).xyz;
        ray.direction = global_to_model * origin_ray.direction;

        vec4 is_inter_hitbox = check_intersect_hitbox(hit_box, ray);

        float distance_to_hitbox = length(is_inter_hitbox.yzw - ray.origin);

        if (is_inter_hitbox.x == 0.0 || distance_to_hitbox > ray.max_distance) {
            offset_indexes += model.indexes_length;
            offset_vertices += model.vertices_length;
            continue;
        }

        for (int i = 0; i < model.indexes_length; i++) {
            uvec3 index = indexes[offset_indexes + i];
            vec3 triangle1 = vertices[offset_vertices + index.x];
            vec3 triangle2 = vertices[offset_vertices + index.y];
            vec3 triangle3 = vertices[offset_vertices + index.z];
            vec3[3] triangles = vec3[](triangle1, triangle2, triangle3);
            IntersectResult res = intersect(ray, triangles);
            if (res.intersect && res.distance < distance && res.distance < ray.max_distance) {
                vec3 normal = normalize(
                    cross(
                        triangle2 - triangle1,
                        triangle3 - triangle1
                    )
                );
                vec3 inter_point = origin_ray.origin + origin_ray.direction.xyz * res.distance;
                distance = res.distance;
                inter = intersection_succ(
                    inter_point,
                    normal,
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

    intersections[idx] = inter;
}