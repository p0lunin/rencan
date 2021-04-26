#version 450

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 1) const uint MSAA_MULTIPLIER = 2;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(set = 1, binding = 0, rgba8) readonly uniform image2D inputImage;

layout(set = 2, binding = 0, rgba8) writeonly uniform image2D resultImage;

void main() {
    uint idx = gl_GlobalInvocationID.x * MSAA_MULTIPLIER;

    ivec2 pixel_pos = ivec2(
        gl_GlobalInvocationID.x % screen.x, gl_GlobalInvocationID.x / screen.x
    );

    uvec2 local_screen = screen * MSAA_MULTIPLIER;

    vec4 color = vec4(0.0);

    for (int i=0; i<MSAA_MULTIPLIER * MSAA_MULTIPLIER; i++) {
        ivec2 local_pixel_pos = ivec2(
            idx % local_screen.x + i % MSAA_MULTIPLIER,
            (idx * MSAA_MULTIPLIER) / local_screen.x + i / MSAA_MULTIPLIER
        );
        color += imageLoad(inputImage, local_pixel_pos);
    }
    color /= MSAA_MULTIPLIER * MSAA_MULTIPLIER;
    imageStore(resultImage, pixel_pos, color);
}
