#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) uniform Camera {
    mat4 cameraToWorld;
    float fov;
};
layout(std140, set = 0, binding = 2) buffer RaysInfo {
    Ray data[];
} rays;
layout(set = 0, binding = 3, rgba8) uniform image2D resultImage;
layout(std140, set = 0, binding = 4) buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 0, binding = 5) buffer DirectLightInfo {
    DirectLight global_light;
};

uint compute_x(uint screen_width) {
    return gl_GlobalInvocationID.x % screen_width;
}

uint compute_y(uint screen_width) {
    return gl_GlobalInvocationID.x / screen_width;
}

void main() {
    uint screen_width = screen.x;
    uint screen_height = screen.y;

    uint idx = gl_GlobalInvocationID.x;

    vec2 this_point = vec2(compute_x(screen_width), compute_y(screen_width));

    float scale = tan(fov / 2);
    float aspect_ratio = float(screen_width) / float(screen_height);

    vec3 origin = (cameraToWorld * vec4(0.0)).xyz;

    float x = (2 * ((this_point.x + 0.5) / float(screen_width)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen_height))) * scale;

    vec4 direction = vec4(
        (inverse(
            mat3(
                cameraToWorld[0].xyz,
                cameraToWorld[1].xyz,
                cameraToWorld[2].xyz
            )
        )) * vec3(x, y, -1.0), 0.0);
    //vec4 direction = cameraToWorld * vec4(x, y, -1.0, 0.0);

    rays.data[idx] = Ray(origin, direction);
}