@compute
@workgroup_size(4, 5, 6)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // v_indices[global_id.x] = collatz_iterations(v_indices[global_id.x]);
}