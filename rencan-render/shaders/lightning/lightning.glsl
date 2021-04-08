#version 450

#extension GL_GOOGLE_include_directive : require

#include "../include/defs.glsl"

const uint SPECULAR_EXPONENT = 200;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(std140, set = 1, binding = 0) readonly buffer Intersections {
    LightRay intersections[];
};

layout(set = 2, binding = 0) writeonly buffer ResultImage {
    uvec4 colors[];
};

layout(std140, set = 3, binding = 0) readonly buffer PreviousIntersections {
    Intersection previous_intersections[];
};

#define PI radians(180)

float compute_specular_component(vec3 primary_ray, vec3 light_ray, vec3 surface_normal) {
    vec3 reflected_ray = reflect(light_ray, surface_normal);
    float specular_component = pow(max(dot(reflected_ray, primary_ray), 0.0), SPECULAR_EXPONENT);
    return specular_component;
}

vec3 compute_specular_color(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, vec3 light_color) {
    float component = compute_specular_component(primary_ray, light_ray, surface_normal);
    return light_color * component;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    LightRay light_int = intersections[idx];
    Intersection inter = previous_intersections[light_int.inter_id];
    if (inter.is_intersect != 1) {
        return;
    }

    vec3 normal = inter.normal;
    mat4 isometry = inter.model.isometry;
    float albedo = inter.model.albedo;

    vec3 light_dir = light_int.ray.direction;

    vec3 color = albedo / PI * light_int.light_intensity * max(dot(normal, light_dir), 0.0);

    color += compute_specular_color(inter.ray.direction, light_dir, inter.normal, light_int.light_intensity);

    uvec4 add_color = uvec4(clamp(color, 0, 1) * 255, 1);
    atomicAdd(colors[inter.pixel_id].x, add_color.x);
    atomicAdd(colors[inter.pixel_id].y, add_color.y);
    atomicAdd(colors[inter.pixel_id].z, add_color.z);
    atomicAdd(colors[inter.pixel_id].w, 1);
}
