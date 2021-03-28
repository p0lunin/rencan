#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

const uint SPECULAR_EXPONENT = 200;

layout(constant_id = 1) const uint SAMPLING = 0;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(std140, set = 1, binding = 0) readonly buffer Rays {
    Ray rays[];
};
layout(std140, set = 1, binding = 1) readonly buffer Intersections {
    Intersection intersections[];
};

layout(std140, set = 2, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 2, binding = 1) readonly buffer ModelInfos {
    ModelInfo[] models;
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

layout(set = 4, binding = 0, rgba8) writeonly uniform image2D resultImage;

#include "include/ray_tracing.glsl"

#define PI radians(180)

vec3 compute_color_for_global_lights(
    vec3 normal,
    Intersection inter,
    Intersection shadow_intersect,
    ModelInfo model,
    Ray primary_ray
) {
    mat4 isometry = model.isometry;
    float albedo = model.albedo;

    vec3 light_dir = -global_light.direction;

    vec3 ray_direction = (inverse(isometry) * primary_ray.direction).xyz;

    vec3 color = albedo / PI * global_light.intensity * global_light.color * max(dot(normal, -global_light.direction.xyz), 0.0);

    return color;
}

vec3 compute_color_for_point_light(
    vec3 normal,
    vec3 light_dir,
    PointLight light,
    float albedo,
    float distance
) {
    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    vec3 color = albedo / PI * intensity * max(dot(normal, -light_dir), 0.0);
    return intensity;
}

Ray make_shadow_ray_for_direction_light(Intersection inter, Ray previous) {
    return Ray(inter.point, vec4(-global_light.direction.xyz, 0.0), 1.0 / 0.0);
}

Ray make_shadow_ray_for_point_light(Intersection inter, Ray previous, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    return Ray(inter.point, vec4(normalize(direction_ray), 0.0), length(direction_ray));
}

float compute_specular_component(vec3 primary_ray, vec3 light_ray, vec3 surface_normal) {
    vec3 reflected_ray = reflect(light_ray, surface_normal);
    float specular_component = pow(max(dot(reflected_ray, primary_ray), 0.0), SPECULAR_EXPONENT);
    return specular_component;
}

vec3 compute_specular_color(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, vec3 light_color) {
    float component = compute_specular_component(primary_ray, light_ray, surface_normal);
    return light_color * component;
}

vec3 compute_color_diffuse_material(ModelInfo model, Intersection inter, Ray primary_ray) {
    vec3 normal = inter.normal;
    Ray global_light_ray = make_shadow_ray_for_direction_light(inter, primary_ray);
    Intersection global_light_inter = trace_first(
        global_light_ray
    );

    vec3 color;

    if (global_light_inter.is_intersect == 0) {
        color = model.diffuse * compute_color_for_global_lights(
            normal,
            inter,
            global_light_inter,
            model,
            primary_ray
        );
        color += model.specular * compute_specular_color(
            primary_ray.direction.xyz,
            global_light_ray.direction.xyz,
            inter.normal,
            global_light.color * global_light.intensity
        );
    }
    else {
        color = vec3(0.0);
    }

    for (int i = 0; i < point_lights_count; i++) {
        PointLight light = point_lights[i];
        vec3 light_dir = light.position - inter.point;
        float distance_to_light = length(light_dir);

        Ray light_ray = make_shadow_ray_for_point_light(
            inter,
            primary_ray,
            light
        );
        Intersection shadow_intersection = trace_first(
            light_ray
        );
        if (shadow_intersection.is_intersect == 1) {
            continue;
        }
        color += model.diffuse * compute_color_for_point_light(
            normal,
            light_dir,
            light,
            model.albedo,
            distance_to_light
        );
        color += model.specular * compute_specular_color(
            primary_ray.direction.xyz,
            light_ray.direction.xyz,
            inter.normal,
            light.color
        );
    }

    return color;
}

vec3 lights(uint idx, Intersection inter, Ray primary_ray) {
    ModelInfo model = models[inter.model_id];

    bool computed = false;
    vec3 color = vec3(0.0);
    for (int i = 0; !computed && i < 100; i ++) {
        switch (model.material) {
            case MATERIAL_DIFFUSE:
                color = compute_color_diffuse_material(model, inter, primary_ray);
                computed = true;
                break;
            case MATERIAL_MIRROR:
                vec3 next_direction = reflect(primary_ray.direction.xyz, inter.normal);
                Ray reflect_ray = Ray(inter.point, vec4(next_direction, 0.0), 1.0 / 0.0);
                Intersection mirror_inter = trace(reflect_ray);
                if (mirror_inter.is_intersect == 0.0) {
                    color = vec3(0.0, 0.7, 0.4);
                    computed = true;
                }
                else {
                    inter = mirror_inter;
                    primary_ray = reflect_ray;
                    model = models[mirror_inter.model_id];
                }
                break;
            default:
                color = vec3(0.0, 0.0, 1.0);
                break;
        }
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

    float x = (2 * ((this_point.x + 0.5) / float(screen.x)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen.y))) * scale;

    vec4 direction = vec4(normalize(camera_rotation * vec3(x, y, -1.0)), 0.0);

    return Ray(origin, direction, 1.0 / 0.0);
}

vec3 compute_color_for_pixel(uint idx, uvec2 screen, uvec2 pixel_pos) {
    Ray primary_ray = compute_primary_ray(
        screen,
        pixel_pos,
        fov,
        pos,
        rotation
    );
    Intersection inter = trace(primary_ray);

    vec3 color;
    if (inter.is_intersect == 1) {
        color = lights(idx, inter, primary_ray);
    }
    else if (inter.is_intersect == 0) {
        color = vec3(0.0, 0.7, 0.4);
    }
    else {
        // unreachable
    }
    return color;
}

vec3 tracing_with_sampling() {
    uint idx = gl_GlobalInvocationID.x * 2;

    uvec2 local_screen = screen * 2;

    vec3 color = vec3(0.0);

    for (int i=0; i<4; i++) {
        uvec2 local_pixel_pos = uvec2(idx % local_screen.x + i % 2, (idx * 2) / local_screen.x + i / 2);
        color += compute_color_for_pixel(idx, local_screen, local_pixel_pos);
    }
    color /= 4;

    return color;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;
    uvec2 pixel_pos = uvec2(idx % screen.x, idx / screen.x);

    vec3 color;

    if (SAMPLING == 1) {
        color = tracing_with_sampling();
    }
    else {
        Ray primary_ray = rays[idx];
        Intersection inter = intersections[idx];

        if (inter.is_intersect == 1) {
            color = lights(idx, inter, primary_ray);
        }
        else if (inter.is_intersect == 0) {
            color = vec3(0.0, 0.7, 0.4);
        }
        else {
            // unreachable
        }
    }
    imageStore(resultImage, ivec2(pixel_pos.xy), vec4(color, 0.0));
}