// #define UNIFORM_BIND_GROUP_Material 0
// #define UNIFORM_BIND_GROUP_DrawCall 1
// #define UNIFORM_BIND_GROUP_Pass 2
// #define UNIFORM_BIND_GROUP_Task 3
// #define UNIFORM_BIND_GROUP_Global 4
// 有无 const 语法？

            // ELightType::Directional => 0,
            // ELightType::Point => 1,
            // ELightType::Spot => 2,

const xx = 0;

struct VertexOut {
    @builtin(position) position : vec4f,

    @location(0) uv0 : vec2f,

    @location(1) normal_world : vec3f,
    @location(2) position_world : vec3f,
    @location(3) object_id: u32,
};

struct FragmentOut {
    @location(0) color : vec4f,
    @location(1) object_id: u32
    // @builtin(frag_depth) depth: f32,
};

struct DrawCallUniform {
    transform_m: mat4x4f,
    transform_mv: mat4x4f,
    transform_mvp: mat4x4f,
    object_id: u32,
};

struct ShaderLight {
    position: vec3f,
    color: vec3f,
    radius: f32,
    direction: vec3f,
    light_type: u32,
};

struct TaskUniform {
    light_count: u32,
    lights: array<ShaderLight, 16>,
};

@group(0) @binding(1) var<uniform> cDrawCall: DrawCallUniform;
@group(0) @binding(3) var<uniform> cTask: TaskUniform;


fn to_vec3f(v: vec4f) -> vec3f {
    return vec3f(v.x, v.y, v.z) / v.w;
}

@vertex
fn vsMain_base(
    @location(0) pos: vec3f, 
    @location(1) normal: vec3f, 
    @location(2) uv: vec2f
) -> VertexOut {
    // cDrawCall.matrix_projection * 
    var vertex_out: VertexOut;
    vertex_out.position = cDrawCall.transform_mvp * vec4f(pos.xyz, 1.0);
    vertex_out.uv0 = uv;

    vertex_out.normal_world = (cDrawCall.transform_m * vec4f(normal, 0.0)).xyz;
    vertex_out.position_world = to_vec3f(cDrawCall.transform_m * vec4f(pos.xyz, 1.0));
    vertex_out.object_id = cDrawCall.object_id;
    return vertex_out;
}

const PI: f32 = 3.14159265358979323846264338327950288;

@fragment
fn fsMain_base(vertex_out: VertexOut) -> FragmentOut {
    // w: linear depth
    // w/z: 0~1, 0 means near

    let albedo: vec3f = vec3f(0.8, 0.8, 0.8);
    let normal = normalize(vertex_out.normal_world);
    let position = vertex_out.position_world;

    var out = vec3f(0.0, 0.0, 0.0); // radiance

    for(var i = 0u; i < cTask.light_count; i++) {
        let light_type = cTask.lights[i].light_type;
        let light_position = cTask.lights[i].position;
        let light_color = cTask.lights[i].color;
        let light_radius = cTask.lights[i].radius;
        if(light_type == 1u) {
            // point light

            let light_to_surface = position - light_position;
            let light_direction = -normalize(light_to_surface);
            // 点光源的 radiance 是个 delta 函数
            // 不知道一般是如何定义一个点光源的。使用基于物理的定义？
            let light_irradiance = light_color / dot(light_to_surface, light_to_surface) * max(0.0, dot(normal, light_direction));
            
            let diffuse = albedo / PI * light_irradiance;   // radiance to every direction
            out += diffuse;
        }
    }


    var fragment_out: FragmentOut;
    fragment_out.color = vec4f(out, 1.0);
    fragment_out.object_id = vertex_out.object_id;
    return fragment_out;
}


@vertex
fn vsMain_model_axis(
    @location(0) pos: vec3f, 
) -> @builtin(position) vec4f {
    return cDrawCall.transform_mvp * vec4f(pos.xyz, 1.0);
}

@fragment
fn fsMain_model_axis() -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0);
}