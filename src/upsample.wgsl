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
var blackout_input_texture: texture_2d<f32>;
@group(0) @binding(1)
var blackout_texture_sampler: sampler;
@group(0) @binding(2)
var input_texture: texture_2d<f32>;
@group(0) @binding(3)
var texture_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSampleLevel(blackout_input_texture, blackout_texture_sampler, in.texcoord, 0.0);
    return col;
}