struct VertexOut {
    @builtin(position) position : vec4f,

    @location(0) uv0 : vec2f,
    @location(1) normal_local : vec3f
};

struct FragmentOut {
    @location(0) color : vec4f,
    @location(1) depth : f32
};

struct DrawCallUniform {
    mat_model: mat4x4f,
    mat_view: mat4x4f,
    mat_proj: mat4x4f,

    // color1: f32,
    // color2: f32,
};

@group(0) @binding(0) var<uniform> cDrawCall: DrawCallUniform;


@vertex
fn vsMain_base(
    @location(0) pos: vec3f, 
    @location(1) normal: vec3f, 
    @location(2) uv: vec2f
) -> VertexOut {
    // cDrawCall.matrix_projection * 
    var vertex_out: VertexOut;
    vertex_out.position = cDrawCall.mat_proj * cDrawCall.mat_view * cDrawCall.mat_model * vec4f(pos.xyz, 1);
    vertex_out.uv0 = uv;
    vertex_out.normal_local = normal;
    
    return vertex_out;
}

@fragment
fn fsMain_base(vertex_out: VertexOut) -> FragmentOut {
    // w: linear depth
    // w/z: 0~1, 0 means near

    var fragment_out: FragmentOut;
    fragment_out.color = vec4f(abs(vertex_out.normal_local.xyz) + 0.3, 1);
    fragment_out.depth = vertex_out.position.w / vertex_out.position.z;
    return fragment_out;
}


@vertex
fn vsMain_depthOnly(
    @location(0) pos: vec3f, 
    @location(1) normal: vec3f, 
    @location(2) uv: vec2f
) -> VertexOut {
    // cDrawCall.matrix_projection * 
    var vertex_out: VertexOut;
    vertex_out.position = cDrawCall.mat_proj * cDrawCall.mat_view * cDrawCall.mat_model * vec4f(pos.xyz, 1);
    vertex_out.uv0 = uv;
    vertex_out.normal_local = normal;
    
    return vertex_out;
}

@fragment
fn fsMain_depthOnly(vertex_out: VertexOut) -> FragmentOut {
    // w: linear depth
    // w/z: 0~1, 0 means near

    var fragment_out: FragmentOut;
    fragment_out.color = vec4f(abs(vertex_out.normal_local.xyz) + 0.3, 1);
    fragment_out.depth = vertex_out.position.w / vertex_out.position.z;
    return fragment_out;
}
