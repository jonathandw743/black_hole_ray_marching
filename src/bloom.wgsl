[[block]]
struct PrimeIndices {
    data: [[stride(4)]] array<u32>;
}; // this is used as both input and output for convenience

@group(0) @binding(0)
var<access(read)> input_texture: texture_2d<f32>;
@group(0) @binding(1)
var<access(write)> output_texture: texture_2d<f32>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;
}