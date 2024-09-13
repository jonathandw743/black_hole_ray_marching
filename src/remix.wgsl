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
var input_texture_0: texture_2d<f32>;
@group(0) @binding(1)
var input_texture_sampler_0: sampler;
@group(0) @binding(2)
var input_texture_1: texture_2d<f32>;
@group(0) @binding(3)
var input_texture_sampler_1: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col_0 = textureSampleLevel(input_texture_0, input_texture_sampler_0, in.texcoord, 0.0);
    let col_1 = textureSampleLevel(input_texture_1, input_texture_sampler_1, in.texcoord, 0.0);
    let col = col_0 + col_1 * 0.5;
    return col;
}