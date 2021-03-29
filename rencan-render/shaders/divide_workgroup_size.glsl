#version 450

#extension GL_GOOGLE_include_directive : require

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 0) const uint DIVIDER = 32;

layout(set = 0, binding = 0) writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint __DO_NOT_TOUCH;
    uint __DO_NOT_TOUCH2;
};

void main() {
    count_intersections /= DIVIDER;
    __DO_NOT_TOUCH = 1;
    __DO_NOT_TOUCH2 = 1;
}