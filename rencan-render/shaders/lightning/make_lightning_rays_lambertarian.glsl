#version 450

#extension GL_GOOGLE_include_directive : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "../include/defs.glsl"

layout(set = 0, binding = 0) readonly buffer Intersections {
    Intersection previous_intersections[];
};

layout(set = 1, binding = 0) writeonly buffer IndirectRays {
    Ray next_rays[];
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

Ray make_shadow_ray_for_direction_light(Intersection inter) {
    vec3 point = inter.point + inter.normal * 0.001;

    Ray ray = Ray(point,-global_light.direction, 1.0 / 0.0);

    return ray;
}

Ray make_shadow_ray_for_point_light(Intersection inter, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    vec3 point = inter.point + inter.normal * 0.001;

    Ray ray = Ray(point, normalize(direction_ray), length(direction_ray));

    return ray;
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