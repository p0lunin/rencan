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
    vec3 origin = camera.pos;
    float proportions = float(screen.x) / float(screen.y) - 1.0;

    uint idx = gl_GlobalInvocationID.x;

    float x = compute_x(screen.x);
    float y = compute_y(screen.x);

    vec2 step = 1.0/(screen - 1);

    vec3 out_direction = vec3(0.0);

    float x_angle = (camera.x_angle * step.x * x) - (camera.x_angle / 2);
    out_direction.x = tan(x_angle);
    float y_angle = (camera.y_angle * step.y * y) - (camera.y_angle / 2);
    out_direction.y = -tan(y_angle);

    out_direction.z = -1.0;

    rays.data[idx] = vec4(
        camera.rotation * out_direction,
        0.0
    );
}