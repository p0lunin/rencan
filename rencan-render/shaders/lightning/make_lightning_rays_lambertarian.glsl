#version 450

#extension GL_GOOGLE_include_directive : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "../include/defs.glsl"

layout(std140, set = 0, binding = 0) readonly buffer Intersections {
    Intersection previous_intersections[];
};

layout(std140, set = 1, binding = 0) writeonly buffer IndirectRays {
    LightRay next_rays[];
};

layout(std140, set = 2, binding = 0) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 2, binding = 1) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 2, binding = 2) readonly buffer PointLights {
    PointLight[] point_lights;
};

layout(set = 3, binding = 0) writeonly buffer RaysCount {
    uint count_rays;
    uint _y_dimension;
    uint _z_dimension;
};

#define PI radians(180)

LightRay make_shadow_ray_for_direction_light(Intersection inter) {
    vec3 point = inter.point + inter.normal * 0.001;

    Ray ray = Ray(point,-global_light.direction, 1.0 / 0.0);

    vec3 intensity = global_light.intensity * global_light.color;

    return LightRay(inter, ray, intensity);
}

LightRay make_shadow_ray_for_point_light(Intersection inter, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    vec3 point = inter.point + inter.normal * 0.001;
    float distance = length(direction_ray);

    Ray ray = Ray(point, normalize(direction_ray), distance);

    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    return LightRay(inter, ray, intensity);
}

uint next_idx() {
    return atomicAdd(count_rays, 1);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];
    ModelInfo model = inter.model;

    if (model.material == MATERIAL_DIFFUSE) {
        next_rays[next_idx()] = make_shadow_ray_for_direction_light(inter);

        for (int i = 0; i < point_lights_count; i++) {
            PointLight light = point_lights[i];
            next_rays[next_idx()] = make_shadow_ray_for_point_light(inter, light);
        }
    }
}