#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "include/defs.glsl"

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(set = 1, binding = 0) writeonly buffer Intersections {
    Intersection intersections[];
};
layout(set = 1, binding = 1) buffer IntersectionsCount {
    uint count_intersections;
    uint __DO_NOT_TOUCH;
    uint __DO_NOT_TOUCH2;
};

layout(std140, set = 2, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 2, binding = 1) readonly buffer ModelInfos {
    ModelInfo models[];
};
layout(set = 2, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 2, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 2, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

#include "include/ray_tracing.glsl"

Ray compute_primary_ray(
    uvec2 screen,
    uvec2 this_point,
    float fov,
    vec3 camera_origin,
    mat3 camera_rotation
) {
    float scale = tan(fov / 2);
    float aspect_ratio = float(screen.x) / float(screen.y);

    vec3 origin = camera_origin;

    float x = (2 * ((this_point.x + 0.5) / float(screen.x)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen.y))) * scale;

    vec4 direction = vec4(normalize(camera_rotation * vec3(x, y, -1.0)), 0.0);

    return Ray(origin, direction, 1.0 / 0.0);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Ray ray = compute_primary_ray(
        screen,
        uvec2(idx % screen.x, idx / screen.x),
        fov,
        pos,
        rotation
    );
    Intersection inter = trace(ray, idx);

    if (inter.is_intersect == 1) {
        uint intersection_idx = atomicAdd(count_intersections, 1);
        intersections[intersection_idx] = inter;
    }
}