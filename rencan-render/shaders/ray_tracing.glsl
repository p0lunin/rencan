#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) buffer Rays {
    Ray rays[];
};
layout(std140, set = 0, binding = 2) buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 1, binding = 0) buffer ModelInfo {
    mat4 isometry;
    uint model_id;
    uint indexes_length;
    float albedo;
};
layout(set = 1, binding = 1) buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 2) buffer Indexes {
    uvec3[] indexes;
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

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    Intersection inter = intersections[idx];

    float distance = 1.0 / 0.0;
    if (inter.is_intersect == 1) {
        distance = inter.distance;
    }

    Ray ray = rays[idx];
    ray.origin = (inverse(isometry) * vec4(ray.origin, 0.0)).xyz;
    ray.direction = vec4((inverse(mat3(isometry[0].xyz, isometry[1].xyz, isometry[2].xyz))) * ray.direction.xyz, 0.0);

    for (int i = 0; i < indexes_length; i++) {
        uvec3 index = indexes[i];
        vec3 triangle1 = vertices[index.x];
        vec3 triangle2 = vertices[index.y];
        vec3 triangle3 = vertices[index.z];
        vec3[3] triangles = vec3[](triangle1, triangle2, triangle3);
        IntersectResult res = intersect(ray, triangles);
        if (res.intersect && res.distance < distance) {
            distance = res.distance;
            inter = intersection_succ(
                model_id, i, res.barycentric_coords, res.distance
            );
        }
    }

    intersections[idx] = inter;
}