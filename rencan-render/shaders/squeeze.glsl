#version 450

const float CONTRAST_THRESHOLD = 0.0833;
const float RELATIVE_THRESHOLD = 0.166;
const float SUBPIXEL_BLENDING = 0.8;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

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

struct Luminance {
    vec3 pos;
    float luminance;
};

// n - north
// e - east
// w - west
// s - sourth
struct LuminanceInfo {
    Luminance center;
    Luminance n, e, s, w;
    Luminance ne, nw, se, sw;
    float highest, lowest, contrast;
};

float rgb_to_luminance(vec3 rgb) {
    float luminance = (rgb.x + rgb.y + rgb.z) / 3;
    return luminance;
}

Luminance load_luminance(ivec2 pos) {
    Luminance luminance;
    luminance.pos = imageLoad(inputImage, pos).xyz;
    luminance.luminance = rgb_to_luminance(luminance.pos);
    return luminance;
}

LuminanceInfo get_luminance_info(ivec2 pos) {
    LuminanceInfo info;
    info.center = load_luminance(pos);

    info.e = load_luminance(ivec2(pos.x + 1, pos.y));
    info.w = load_luminance(ivec2(pos.x - 1, pos.y));
    info.n = load_luminance(ivec2(pos.x, pos.y + 1));
    info.s = load_luminance(ivec2(pos.x, pos.y - 1));

    info.ne = load_luminance(ivec2(pos.x + 1, pos.y + 1));
    info.nw = load_luminance(ivec2(pos.x - 1, pos.y + 1));
    info.se = load_luminance(ivec2(pos.x + 1, pos.y - 1));
    info.sw = load_luminance(ivec2(pos.x - 1, pos.y - 1));

    info.lowest = min(info.n.luminance, min(info.e.luminance, min(info.w.luminance, info.s.luminance)));
    info.highest = max(info.n.luminance, max(info.e.luminance, max(info.w.luminance, info.s.luminance)));
    info.contrast = info.highest - info.lowest;

    return info;
}

float calc_factor(LuminanceInfo l) {
    float f = 2 * (l.n.luminance + l.e.luminance + l.s.luminance + l.w.luminance);
    f += l.ne.luminance + l.nw.luminance + l.se.luminance + l.sw.luminance;
    f *= 1.0 / 12;
    f = abs(f - l.center.luminance);
    f = clamp(f / l.contrast, 0.0, 1.0);
    float factor = smoothstep(0, 1, f);
    factor = factor * factor;
    return factor * SUBPIXEL_BLENDING;
}

const uint EDGE_HORIZONTAL = 1;
const uint EDGE_VERTICAL = 2;

struct EdgeInfo {
    uint direction;
    int step;
    float opposite_luminance, gradient;
};

EdgeInfo calc_edge(LuminanceInfo l) {
    EdgeInfo info;

    float horizontal =
        abs(l.n.luminance + l.s.luminance - 2 * l.center.luminance) * 2 +
        abs(l.ne.luminance + l.se.luminance - 2 * l.e.luminance) +
        abs(l.nw.luminance + l.sw.luminance - 2 * l.w.luminance);
    float vertical =
        abs(l.e.luminance + l.w.luminance - 2 * l.center.luminance) * 2 +
        abs(l.ne.luminance + l.nw.luminance - 2 * l.n.luminance) +
        abs(l.se.luminance + l.sw.luminance - 2 * l.s.luminance);
    info.direction = horizontal >= vertical ? EDGE_HORIZONTAL : EDGE_VERTICAL;

    float p_luminance = info.direction == EDGE_HORIZONTAL ? l.n.luminance : l.e.luminance;
    float n_luminance = info.direction == EDGE_HORIZONTAL ? l.s.luminance : l.w.luminance;

    float p_gradient = abs(p_luminance - l.center.luminance);
    float n_gradient = abs(n_luminance - l.center.luminance);

    if (p_gradient < n_gradient) {
        info.step = -1;
        info.opposite_luminance = n_luminance;
        info.gradient = n_gradient;
    }
    else {
        info.step = 1;
        info.opposite_luminance = p_luminance;
        info.gradient = p_gradient;
    }
    return info;
}

float determine_edge_blend_factor(LuminanceInfo l, EdgeInfo e, ivec2 pos) {
    return e.gradient;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (pos.x > screen.x / 2) {
        imageStore(resultImage, pos, imageLoad(inputImage, pos));
        return;
    }

    LuminanceInfo info = get_luminance_info(pos);

    float threshold = max(CONTRAST_THRESHOLD, RELATIVE_THRESHOLD * info.highest);

    if (info.contrast < threshold) {
        imageStore(resultImage, pos, imageLoad(inputImage, pos));
        return;
    }

    float factor = calc_factor(info);

    EdgeInfo edge = calc_edge(info);

    vec3 out_color;
    if (edge.direction == EDGE_HORIZONTAL) {
        out_color = edge.step == 1 ?
                info.center.pos * info.center.luminance +
                    info.n.pos * factor * 0.5 +
                    info.ne.pos * factor * 0.25 +
                    info.nw.pos * factor * 0.25 :
                info.center.pos * info.center.luminance +
                    info.s.pos * factor * 0.5 +
                    info.se.pos * factor * 0.25 +
                    info.sw.pos * factor * 0.25 ;
        /*float rest_factor = factor;
        factor = (1 - factor);
        out_color = edge.step == 1 ?
                info.center.pos * factor +
                    info.n.pos * rest_factor * 0.5 +
                    info.ne.pos * rest_factor * 0.25 +
                    info.nw.pos * rest_factor * 0.25 :
                info.center.pos * factor +
                    info.s.pos * rest_factor * 0.5 +
                    info.se.pos * rest_factor * 0.25 +
                    info.sw.pos * rest_factor * 0.25 ;*/
    }
    else {
        float rest_factor = factor;
        factor = (1 - factor);
        out_color = edge.step == 1 ?
                info.center.pos * factor +
                    info.e.pos * rest_factor * 0.5 +
                    info.ne.pos * rest_factor * 0.25 +
                    info.se.pos * rest_factor * 0.25 :
                info.center.pos * factor +
                    info.w.pos * rest_factor * 0.5 +
                    info.sw.pos * rest_factor * 0.25 +
                    info.nw.pos * rest_factor * 0.25 ;
    }
    //out_color = vec3(determine_edge_blend_factor(info, edge, pos));

    imageStore(resultImage, pos, vec4(out_color, 0.0));
}
