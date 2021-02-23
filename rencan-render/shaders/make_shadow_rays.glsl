#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly buffer PreviousRays {
    Ray previous_rays[];
};
layout(std140, set = 0, binding = 2) buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 0, binding = 3) buffer NextRays {
    Ray next_rays[];
};
layout(std140, set = 0, binding = 4) readonly uniform DirectLightInfo {
    DirectLight global_light;
};

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    Intersection inter = intersections[idx];
    Ray ray = previous_rays[idx];
    if (inter.is_intersect == 0) {
        return;
    }

    vec3 point = ray.origin + ray.direction.xyz * inter.distance + normalize(-global_light.direction.xyz) * 0.001;

    next_rays[idx] = Ray(point, vec4(-global_light.direction, 0.0));
}