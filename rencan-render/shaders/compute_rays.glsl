#version 450

struct Ray {
    vec3 origin;
    vec4 direction;
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) uniform Camera {
    vec3 pos;
    mat3 rotation;
    float x_angle;
    float y_angle;
} camera;
layout(std140, set = 0, binding = 2) buffer RaysInfo {
    Ray data[];
} rays;
layout(set = 0, binding = 3, rgba8) uniform image2D resultImage;

uint compute_x(uint screen_width) {
    return gl_GlobalInvocationID.x % screen_width;
}

uint compute_y(uint screen_width) {
    return gl_GlobalInvocationID.x / screen_width;
}

void main() {
    uint screen_width = screen.x;
    uint screen_height = screen.y;
    float proportions = float(screen_width) / float(screen_height);
    float x_props;

    if (proportions >= 1.0) {
        x_props = proportions;
    }
    else {
        x_props = 1.0;
    }

    uint idx = gl_GlobalInvocationID.x;

    float x = compute_x(screen_width);
    float y = compute_y(screen_width);

    float step_x = x_props/(screen_width - 1);
    float step_y = 1.0/(screen_height - 1);

    float max_x_offset = screen_width - screen_height;
    if (max_x_offset < 0.0) max_x_offset = 0.0;
    max_x_offset = max_x_offset / screen_width;

    float max_deviation_x = tan(camera.x_angle/2);
    float max_deviation_y = tan(camera.y_angle/2);

    float x_deviation_local = step_x * x * (2 * proportions) * max_deviation_x - max_deviation_x;
    float y_deviation_local = -(step_y * y * 2 * max_deviation_y) + max_deviation_y;

    vec3 deviation_global = camera.rotation * vec3(x_deviation_local - max_x_offset * x_deviation_local, y_deviation_local, -1.0);

    vec4 ray_direction = vec4(
        deviation_global,
        0.0
    );

    vec3 add_to_origin = camera.rotation * vec3(
        max_x_offset * x_deviation_local,
        0.0,
        0.0
    );

    vec3 origin = camera.pos + add_to_origin;

    rays.data[idx] = Ray(origin, ray_direction);
}