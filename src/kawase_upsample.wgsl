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
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var<uniform> resolution: vec2u;

// https://www.shadertoy.com/view/3td3W8
@fragment
fn main( in: VertexOutput ) -> @location(0) vec4f
{
    // vec2 uv = vec2(fragCoord.xy / (iResolution.xy * 2.0));
    let uv = in.texcoord / 2.0;
    // vec2 halfpixel = 0.5 / (iResolution.xy * 2.0);
    let halfpixel = 0.5 / (vec2f(resolution) * 2.0);
    let offset = 3.0;

    var sum = textureSample(input_texture, texture_sampler, uv + vec2(-halfpixel.x * 2.0, 0.0) * offset);
    
    sum += textureSample(input_texture, texture_sampler, uv + vec2(-halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += textureSample(input_texture, texture_sampler, uv + vec2(0.0, halfpixel.y * 2.0) * offset);
    sum += textureSample(input_texture, texture_sampler, uv + vec2(halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += textureSample(input_texture, texture_sampler, uv + vec2(halfpixel.x * 2.0, 0.0) * offset);
    sum += textureSample(input_texture, texture_sampler, uv + vec2(halfpixel.x, -halfpixel.y) * offset) * 2.0;
    sum += textureSample(input_texture, texture_sampler, uv + vec2(0.0, -halfpixel.y * 2.0) * offset);
    sum += textureSample(input_texture, texture_sampler, uv + vec2(-halfpixel.x, -halfpixel.y) * offset) * 2.0;

    return sum / 12.0;
}