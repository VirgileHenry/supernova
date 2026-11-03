#version 460

struct CameraUniform {
    mat4 view;
    mat4 proj;
};

struct SegmentInstance {
    mat4 transform;
    mat4 transform_inv;
    vec4 aabb_min;
    vec4 aabb_max;
    uint shell_offset;
    uint shell_length;
    uint interior_offset;
    uint interior_length;
};

struct ScreenSpaceQuad {
    vec2 min;
    vec2 max;
};

// Camera uniform buffer
layout(set = 0, binding = 0, std140) uniform CameraBuffer {
    CameraUniform camera;
};

// Instance storage buffer
layout(set = 0, binding = 1, std430) readonly buffer InstanceBuffer {
    SegmentInstance instances[];
};

// Pass the instance index to the fragment shader
layout(location = 0) flat out uint out_instance_index;
// Pass the ndc to the fragment shader
layout(location = 1) out vec2 out_ndc;

vec2 corner_to_ndc(SegmentInstance instance, vec3 corner) {
    vec4 local_pos = vec4(corner, 1.0);
    vec4 world_pos = instance.transform * local_pos;
    vec4 cam_view_pos = camera.view * world_pos;
    vec4 clip_pos = camera.proj * cam_view_pos;

    // Perspective divide: clip space -> NDC.
    vec2 ndc = clip_pos.xy / clip_pos.w;
    return ndc;
}

void main() {
    // Generate a screen space quad frome the bounding box:
    //   0 ---- 1/4
    //   |    /  |
    //   |   /   |
    //   |  /    |
    //  2/3 ---- 5
    // the gl_VertexIndex tells which vertex we should generate,
    // the gl_InstanceIndex tells which instance data to use.

    SegmentInstance instance = instances[gl_InstanceIndex];

    vec2 debug_ndc = corner_to_ndc(instance, vec3(0.0));

    // The 8 corners of the local-space AABB.
    vec3 corners[8] = vec3[](
            vec3(instance.aabb_min.x, instance.aabb_min.y, instance.aabb_min.z),
            vec3(instance.aabb_max.x, instance.aabb_min.y, instance.aabb_min.z),
            vec3(instance.aabb_min.x, instance.aabb_max.y, instance.aabb_min.z),
            vec3(instance.aabb_max.x, instance.aabb_max.y, instance.aabb_min.z),

            vec3(instance.aabb_min.x, instance.aabb_min.y, instance.aabb_max.z),
            vec3(instance.aabb_max.x, instance.aabb_min.y, instance.aabb_max.z),
            vec3(instance.aabb_min.x, instance.aabb_max.y, instance.aabb_max.z),
            vec3(instance.aabb_max.x, instance.aabb_max.y, instance.aabb_max.z)
        );

    // Use the first corner to initialize bounding quad
    vec2 first_corner_ndc = corner_to_ndc(instance, corners[0]);
    ScreenSpaceQuad bounding_quad = ScreenSpaceQuad(first_corner_ndc, first_corner_ndc);

    // Grow the bounding quad with each other corners
    for (int i = 1; i < 8; i++) {
        vec2 corner_ndc = corner_to_ndc(instance, corners[i]);
        bounding_quad.min = min(bounding_quad.min, corner_ndc);
        bounding_quad.max = max(bounding_quad.max, corner_ndc);
    }

    // Four corners of the screen-space quad.
    vec2 quad_corners[4] = vec2[](
            vec2(bounding_quad.min.x, bounding_quad.min.y), // 0: bottom-left
            vec2(bounding_quad.max.x, bounding_quad.min.y), // 1: bottom-right
            vec2(bounding_quad.min.x, bounding_quad.max.y), // 2: top-left
            vec2(bounding_quad.max.x, bounding_quad.max.y) // 3: top-right
        );

    // Two triangles: 0,1,2 and 2,1,3.
    const int indices[6] = int[](0, 1, 2, 2, 1, 3);

    vec2 quad_pos = quad_corners[indices[gl_VertexIndex]];

    out_instance_index = gl_InstanceIndex;
    out_ndc = quad_pos;

    // Directly output NDC XY as clip-space XY.
    // Z is a temporary placeholder.
    gl_Position = vec4(quad_pos, 0.5, 1.0);
}
