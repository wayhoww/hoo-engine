// #define UNIFORM_BIND_GROUP_Material 0
// #define UNIFORM_BIND_GROUP_DrawCall 1
// #define UNIFORM_BIND_GROUP_Pass 2
// #define UNIFORM_BIND_GROUP_Task 3
// #define UNIFORM_BIND_GROUP_Global 4

const xx = 0;

struct VertexOut {
    @builtin(position) position : vec4f,
    @location(0) uv0 : vec2f,
};

struct FragmentOut {
    @location(0) color : vec4f,
};


fn to_vec3f(v: vec4f) -> vec3f {
    return vec3f(v.x, v.y, v.z) / v.w;
}

@vertex
fn vsMain_base(
    @location(0) pos: vec3f, 
    @location(1) uv: vec2f
) -> VertexOut { 
    var vertex_out: VertexOut;
    vertex_out.position = vec4f(pos.xyz, 1.0);
    vertex_out.uv0 = uv;
    return vertex_out;
}

const PI: f32 = 3.14159265358979323846264338327950288;

@fragment
fn fsMain_base(vertex_out: VertexOut) -> FragmentOut {
    // w: linear depth
    // w/z: 0~1, 0 means near
    var fragment_out: FragmentOut;

    fragment_out.color = vec4f(vertex_out.uv0.xy, 0.0, 1.0);
    
    return fragment_out;
}