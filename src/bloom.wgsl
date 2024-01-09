@group(0) @binding(0)
var<access(read)> input_texture: texture_2d<f32>;
@group(0) @binding(1)
var<access(write)> output_texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;
    let fpos = vec3<f32>(pos);
    textureStore(0, pos, textureSample(input_texture, texture_sampler, fpos / 2.0));
}