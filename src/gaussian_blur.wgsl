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
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let col_from_prev_texture = textureSampleLevel(prev_texture, prev_texture_sampler, in.texcoord, 0.0);
    // let col_from_original_texture = textureSampleLevel(original_texture, original_texture_sampler, in.texcoord, 0.0);
    // let col_from_blackout_texture = textureSampleLevel(blackout_texture, blackout_texture_sampler, in.texcoord, 0.0);
    // let col = col_from_original_texture + col_from_blackout_texture;
    var col = vec4f(0.0);
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.02, -0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.01, -0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.00, -0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.01, -0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.02, -0.02));
    
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.02, -0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.01, -0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.00, -0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.01, -0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.02, -0.01));
    
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.02, -0.00));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.01, -0.00));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.00, -0.00));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.01, -0.00));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.02, -0.00));

    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.02, 0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.01, 0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.00, 0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.01, 0.01));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.02, 0.01));

    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.02, 0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.01, 0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(-0.00, 0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.01, 0.02));
    col += textureSample(texture, texture_sampler, in.texcoord + vec2f(0.02, 0.02));

    col /= 25.0;
    
    return col;
}