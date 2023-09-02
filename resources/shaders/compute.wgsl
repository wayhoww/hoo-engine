struct DrawCallUniform {
    mouse_position: vec2u,
};

@group(0) @binding(1) var<uniform> cDrawCall: DrawCallUniform;

@group(1) @binding(0) var<storage, read_write> storeBuf: array<u32>;
@group(1) @binding(1) var TexObjectID: texture_2d<u32>;
// @group(1) @binding(1) var SamLinearSampler: sampler;
// @group(1) @binding(3) var storeTex : texture_storage_2d<rgba32float, write>;

@compute
@workgroup_size(4, 5, 6)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    /*
    fn textureStore(t: texture_storage_2d<F,write>,
                coords: vec2<C>,
                value: vec4<CF>)
    */
    storeBuf[0] = textureLoad(TexObjectID, cDrawCall.mouse_position, 0).x;
    // storeBuf[0] = cDrawCall.mouse_position[0];
    // storeBuf[1] = cDrawCall.mouse_position[1];
    // textureStore(storeTex, global_id.xy, vec4f(1.0, 2.0, 3.0, 4.0));
}
