#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "../include/defs.glsl"

layout(std140, set = 0, binding = 0) writeonly buffer Rays {
    LightRay not_intersected_rays[];
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

layout(std140, set = 2, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 2, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 2, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
};

layout(std140, set = 3, binding = 0) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 3, binding = 1) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 3, binding = 2) readonly buffer PointLights {
    PointLight[] point_lights;
};

layout(set = 4, binding = 0) writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint _y_dimension;
    uint _z_dimension;
};

layout(std140, set = 5, binding = 0) readonly buffer Intersections {
    Intersection previous_intersections[];
};

#include "../include/ray_tracing.glsl"

LightRay make_shadow_ray_for_direction_light(
    vec3 inter_point,
    vec3 inter_normal
) {
    vec3 point = inter_point + inter_normal * 0.001;

    Ray ray = Ray(point, -global_light.direction, 1.0 / 0.0);

    vec3 intensity = global_light.intensity * global_light.color;

    return LightRay(ray, intensity, gl_GlobalInvocationID.x);
}

LightRay make_shadow_ray_for_point_light(
    vec3 inter_point,
    vec3 inter_normal,
    PointLight light
) {
    vec3 direction_ray = light.position - inter_point;

    vec3 point = inter_point + inter_normal * 0.001;
    float distance = length(direction_ray);

    Ray ray = Ray(point, normalize(direction_ray), distance);

    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    return LightRay(ray, intensity, gl_GlobalInvocationID.x);
}

uint next_idx() {
    return atomicAdd(count_intersections, 1);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];

    if (inter.model_material.material == MATERIAL_DIFFUSE) {
        if (global_light.intensity > 0.01) {
            LightRay global_ray = make_shadow_ray_for_direction_light(inter.point, inter.normal);

            if (!trace_any(global_ray.ray)) {
                not_intersected_rays[next_idx()] = global_ray;
            }
        }

        PointLight light;
        LightRay point_light_ray;

        for (int i = 0; i < point_lights_count; i++) {
            light = point_lights[i];
            point_light_ray = make_shadow_ray_for_point_light(inter.point, inter.normal, light);
            if (!trace_any(point_light_ray.ray)) {
                not_intersected_rays[next_idx()] = point_light_ray;
            }
        }
    }
}