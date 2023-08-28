
@group(0) @binding(2) var storeTex : texture_storage_2d<rgba32float, write>;

@compute
@workgroup_size(4, 5, 6)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    /*
    fn textureStore(t: texture_storage_2d<F,write>,
                coords: vec2<C>,
                value: vec4<CF>)
    */
    textureStore(storeTex, global_id.xy, vec4f(1.0, 2.0, 3.0, 4.0));
}