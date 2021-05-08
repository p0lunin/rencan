#version 450

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(set = 1, binding = 0) uniform sampler2D inputImage;

layout(set = 2, binding = 0, rgba8) writeonly uniform image2D resultImage;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    ivec2 pixel_pos = ivec2(idx % screen.x, idx / screen.x);
    vec2 pixel_coord = vec2(pixel_pos) / vec2(screen);
    vec2 texel_step = vec2(1.0) / vec2(screen);

    ivec2 offset[25];
    offset[0] = ivec2(-2,-2);
    offset[1] = ivec2(-1,-2);
    offset[2] = ivec2(0,-2);
    offset[3] = ivec2(1,-2);
    offset[4] = ivec2(2,-2);

    offset[5] = ivec2(-2,-1);
    offset[6] = ivec2(-1,-1);
    offset[7] = ivec2(0,-1);
    offset[8] = ivec2(1,-1);
    offset[9] = ivec2(2,-1);

    offset[10] = ivec2(-2,0);
    offset[11] = ivec2(-1,0);
    offset[12] = ivec2(0,0);
    offset[13] = ivec2(1,0);
    offset[14] = ivec2(2,0);

    offset[15] = ivec2(-2,1);
    offset[16] = ivec2(-1,1);
    offset[17] = ivec2(0,1);
    offset[18] = ivec2(1,1);
    offset[19] = ivec2(2,1);

    offset[20] = ivec2(-2,2);
    offset[21] = ivec2(-1,2);
    offset[22] = ivec2(0,2);
    offset[23] = ivec2(1,2);
    offset[24] = ivec2(2,2);


    float kernel[25];
    kernel[0] = 1.0f/256.0f;
    kernel[1] = 1.0f/64.0f;
    kernel[2] = 3.0f/128.0f;
    kernel[3] = 1.0f/64.0f;
    kernel[4] = 1.0f/256.0f;

    kernel[5] = 1.0f/64.0f;
    kernel[6] = 1.0f/16.0f;
    kernel[7] = 3.0f/32.0f;
    kernel[8] = 1.0f/16.0f;
    kernel[9] = 1.0f/64.0f;

    kernel[10] = 3.0f/128.0f;
    kernel[11] = 3.0f/32.0f;
    kernel[12] = 9.0f/64.0f;
    kernel[13] = 3.0f/32.0f;
    kernel[14] = 3.0f/128.0f;

    kernel[15] = 1.0f/64.0f;
    kernel[16] = 1.0f/16.0f;
    kernel[17] = 3.0f/32.0f;
    kernel[18] = 1.0f/16.0f;
    kernel[19] = 1.0f/64.0f;

    kernel[20] = 1.0f/256.0f;
    kernel[21] = 1.0f/64.0f;
    kernel[22] = 3.0f/128.0f;
    kernel[23] = 1.0f/64.0f;
    kernel[24] = 1.0f/256.0f;

    vec3 sum = vec3(0.0);
    float c_phi = 1.0;
    float n_phi = 0.5;
	vec3 cval = clamp(texture(inputImage, pixel_coord).xyz, 0, 1);

    float cum_w = 0.0;
    for (int i = 0; i<25; i++) {
        vec2 uv = pixel_coord + texel_step * offset[i] * 1.5;

        vec3 ctmp = clamp(texture(inputImage, uv).xyz, 0, 1);
        vec3 t = cval - ctmp;
        float dist2 = dot(t,t);
        float c_w = min(exp(-(dist2)/c_phi), 1.0);

        float weight = c_w;
        sum += ctmp*weight*kernel[i];
        cum_w += weight*kernel[i];
    }

    vec3 color = sum/cum_w;
    vec3 out_color = cval * 0.3 + color * 0.7;
    imageStore(resultImage, pixel_pos, vec4(color, 1.0));
}
