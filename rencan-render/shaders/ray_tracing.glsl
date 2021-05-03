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

layout(std140, set = 1, binding = 0) writeonly buffer Intersections {
    Intersection intersections[];
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

layout(std140, set = 3, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 3, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 3, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
};

layout(std140, set = 4, binding = 0) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 4, binding = 1) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 4, binding = 2) readonly buffer PointLights {
    PointLight[] point_lights;
};

layout(set = 5, binding = 0) writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint _y_dimension;
    uint _z_dimension;
};

layout(set = 6, binding = 0, rgba8) writeonly uniform image2D resultImage;

layout(push_constant) readonly uniform Offsets {
    uint offset;
    uint msaa;
} offsets;

#include "include/ray_tracing.glsl"

LightRay make_shadow_ray_for_direction_light(
    vec3 inter_point,
    vec3 inter_normal,
    uint idx
) {
    vec3 point = inter_point + inter_normal * 0.001;

    Ray ray = Ray(point, -global_light.direction, 1.0 / 0.0);

    vec3 intensity = global_light.intensity * global_light.color;

    return LightRay(ray, intensity, idx);
}

LightRay make_shadow_ray_for_point_light(
    vec3 inter_point,
    vec3 inter_normal,
    PointLight light,
    uint idx
) {
    vec3 direction_ray = light.position - inter_point;

    vec3 point = inter_point + inter_normal * 0.001;
    float distance = length(direction_ray);

    Ray ray = Ray(point, normalize(direction_ray), distance);

    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    return LightRay(ray, intensity, idx);
}

vec3 compute_color_diffuse_material(Intersection inter) {
    LightRay global_light_ray = make_shadow_ray_for_direction_light(inter.point, inter.normal, inter.pixel_id);
    bool is_global_light_inter = trace_any(global_light_ray.ray);

    vec3 color;

    if (is_global_light_inter) {
        color = compute_light_color(
            inter.model_material,
            global_light_ray.light_intensity,
            inter.normal,
            global_light_ray.ray.direction,
            inter.ray.direction
        );
    }
    else {
        color = vec3(0.0);
    }

    for (int i = 0; i < point_lights_count; i++) {
        PointLight light = point_lights[i];
        vec3 light_dir = light.position - inter.point;
        float distance_to_light = length(light_dir);

        LightRay light_ray = make_shadow_ray_for_point_light(
            inter.point,
            inter.normal,
            light,
            inter.pixel_id
        );
        if (trace_any(light_ray.ray)) {
            continue;
        }
        color += compute_light_color(
            inter.model_material,
            light_ray.light_intensity,
            inter.normal,
            light_ray.ray.direction,
            inter.ray.direction
        );
    }

    return color;
}

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

    float x = (2 * ((this_point.x + 0.5) / float(screen.x * offsets.msaa)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen.y * offsets.msaa))) * scale;

    vec3 direction = normalize(camera_rotation * vec3(x, y, -1.0));

    return Ray(origin, direction, 1.0 / 0.0);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Ray ray = compute_primary_ray(
        screen,
        uvec2((offsets.offset + idx) % (screen.x * offsets.msaa), (offsets.offset + idx) / (screen.x * offsets.msaa)),
        fov,
        pos,
        rotation
    );
    Intersection inter;
    bool is_inter = trace(ray, idx, inter);

    bool computed = false;
    vec3 color = vec3(0.0);
    for (int i = 0; is_inter && !computed && i < 100; i++) {
        switch (inter.model_material.material) {
            case MATERIAL_DIFFUSE:
                color = compute_color_diffuse_material(inter);
                computed = true;
                is_inter = true;
                break;
            case MATERIAL_MIRROR:
                vec3 next_direction = reflect(inter.ray.direction, inter.normal);
                Ray reflect_ray = Ray(inter.point, next_direction, 1.0 / 0.0);
                Intersection mirror_inter;
                bool is_mirror_inter = trace(reflect_ray, idx, mirror_inter);
                if (!is_mirror_inter) {
                    computed = true;
                    is_inter = false;
                }
                else {
                    inter = mirror_inter;
                }
                break;
        }
    }

    if (computed && is_inter && inter.model_material.material == MATERIAL_DIFFUSE) {
        uint intersection_idx = atomicAdd(count_intersections, 1);
        intersections[intersection_idx] = inter;

        imageStore(
            resultImage,
            ivec2(
                (offsets.offset + idx) % (screen.x * offsets.msaa),
                (offsets.offset + idx) / (screen.x * offsets.msaa)
            ),
            vec4(color, 1.0)
        );
    }
}