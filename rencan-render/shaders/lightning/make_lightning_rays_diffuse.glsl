#version 450

#extension GL_GOOGLE_include_directive : require

layout(constant_id = 1) const uint MAX_INDIRECT_RAYS = 32;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "include/defs.glsl"

layout(set = 0, binding = 0) readonly buffer Rays {
    Ray previous_rays[];
};
layout(set = 0, binding = 1) readonly buffer Intersections {
    Intersection previous_intersections[];
};

layout(set = 1, binding = 0) writeonly buffer IndirectRays {
    LightningRay next_rays[];
};
layout(set = 1, binding = 1) readonly buffer IndirectIntersections {
    Intersection next_intersections[];
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

layout(std140, set = 3, binding = 0) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 3, binding = 1) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 3, binding = 2) readonly buffer PointLights {
    PointLight[] point_lights;
};

LightningRay make_shadow_ray_for_direction_light(Intersection inter) {
    vec3 point = inter.point + inter.normal * 0.001;

    Ray ray = Ray(point, vec4(-global_light.direction.xyz, 0.0), 1.0 / 0.0);

    return LightningRay(ray, RAY_TYPE_SHADING);
}

LightningRay make_shadow_ray_for_point_light(Intersection inter, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    vec3 point = inter.point + inter.normal * 0.001;

    Ray ray = Ray(point, vec4(normalize(direction_ray), 0.0), length(direction_ray));

    return LightningRay(ray, RAY_TYPE_SHADING);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];
    ModelInfo model = models[inter.model_id];

    if (inter.is_intersect == 1 && model.material == MATERIAL_DIFFUSE) {
        uint offset = idx * MAX_INDIRECT_RAYS;
        next_rays[offset] = make_shadow_ray_for_direction_light(inter);

        for (int i = 0; i + 1 < MAX_INDIRECT_RAYS && i < point_lights_count; i++) {
            PointLight light = point_lights[i];
            next_rays[offset + i + 1] = make_shadow_ray_for_point_light(inter, light);
        }
    }
}