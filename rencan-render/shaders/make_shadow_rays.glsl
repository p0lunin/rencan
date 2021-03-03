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
layout(std140, set = 0, binding = 2) readonly buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 0, binding = 3) writeonly buffer NextRays {
    Ray next_rays[];
};
layout(std140, set = 0, binding = 4) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 0, binding = 5) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 0, binding = 6) readonly buffer PointLights {
    PointLight[] point_lights;
};

Ray make_shadow_ray_for_direction_light(Intersection inter, Ray previous) {
    vec3 direction_ray = -global_light.direction.xyz;

    vec3 point = previous.origin + previous.direction.xyz * inter.distance + direction_ray * 0.001;

    return Ray(point, vec4(direction_ray, 0.0));
}

Ray make_shadow_ray_for_point_light(Intersection inter, Ray previous, PointLight light) {
    vec3 inter_point = previous.origin + previous.direction.xyz * inter.distance;
    vec3 direction_ray = light.position - inter_point;

    vec3 point = inter_point + (-previous.direction.xyz) * 0.001;

    return Ray(point, vec4(direction_ray, 0.0));
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    Intersection inter = intersections[idx];
    Ray ray = previous_rays[idx];
    if (inter.is_intersect == 1) {
        next_rays[idx] = make_shadow_ray_for_direction_light(inter, ray);
    }

    for (int i = 0; i < point_lights_count; i++) {
        PointLight light = point_lights[i];
        uint offset = (i + 1) * screen.x * screen.y;

        next_rays[offset + idx] = make_shadow_ray_for_point_light(inter, ray, light);
    }
}