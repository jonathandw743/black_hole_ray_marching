struct VertexOutput {
    // Mark output position as invariant so it's safe to use it with depth test Equal.
    // Without @invariant, different usages in different render pipelines might optimize differently,
    // causing slightly different results.
    @invariant @builtin(position)
    position: vec4f,
    @location(0)
    texcoord: vec2f,
};

@group(0) @binding(0)
var prev_texture: texture_2d<f32>;
@group(0) @binding(1)
var prev_texture_sampler: sampler;
@group(0) @binding(2)
var original_blackout_texture: texture_2d<f32>;
@group(0) @binding(3)
var original_blackout_texture_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_from_prev_texture = textureSampleLevel(prev_texture, prev_texture_sampler, in.texcoord, 0.0);
    let col_from_original_blackout_texture = textureSampleLevel(original_blackout_texture, original_blackout_texture_sampler, in.texcoord, 0.0);
    let col = mix(col_from_prev_texture, col_from_original_blackout_texture, 0.1);
    return col;
}