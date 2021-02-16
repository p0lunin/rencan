#version 450

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
layout(set = 0, binding = 2) buffer RaysInfo {
    vec4 data[];
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
    float proportions = float(screen_width) / float(screen_height) - 1.0;

    uint idx = gl_GlobalInvocationID.x;

    float x = compute_x(screen_width);
    float y = compute_y(screen_width);

    float step_x = 1.0/(screen_width - 1);
    float step_y = 1.0/(screen_height - 1);

    float max_deviation_x = tan(camera.x_angle/2);
    float max_deviation_y = tan(camera.y_angle/2);

    float x_deviation_local = step_x * x * (2*max_deviation_x*(1 + proportions)) - (max_deviation_x + proportions);
    float y_deviation_local = -(step_y * y * 2 * max_deviation_y) + max_deviation_y;

    vec3 deviation_global = camera.rotation * vec3(x_deviation_local, y_deviation_local, -1.0);

    vec4 ray_direction = vec4(
        deviation_global,
        0.0
    );

    rays.data[idx] = ray_direction;
}