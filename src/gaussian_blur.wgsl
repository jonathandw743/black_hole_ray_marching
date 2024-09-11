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

@group(0) @binding(2)
var<uniform> resolution: vec2u;

const kernel_size: f32 = 10.0;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let col_from_prev_texture = textureSampleLevel(prev_texture, prev_texture_sampler, in.texcoord, 0.0);
    // let col_from_original_texture = textureSampleLevel(original_texture, original_texture_sampler, in.texcoord, 0.0);
    // let col_from_blackout_texture = textureSampleLevel(blackout_texture, blackout_texture_sampler, in.texcoord, 0.0);
    // let col = col_from_original_texture + col_from_blackout_texture;
    let pixel_size = vec2f(1.0 / f32(resolution.x), 1.0 / f32(resolution.y));

    var col = vec4f(0.0);
    var total_multiplier = 0.0;
    for (var x = -kernel_size; x <= kernel_size; x += 1.0) {
        for (var y = -kernel_size; y <= kernel_size; y += 1.0) {
            let offset = vec2f(x, y);
            let d = length(offset);
            let s = d / (kernel_size + 1.0);
            var multiplier = 3.0 * (1.0 - s) * (1.0 - s) - 2.0 * (1.0 - s) * (1.0 - s) * (1.0 - s);
            if s > 1.0 {
                multiplier = 0.0;
            }
            let scaled_offset = vec2f(pixel_size.x * offset.x, pixel_size.y * offset.y);
            col += textureSample(texture, texture_sampler, in.texcoord + scaled_offset) * multiplier;
            total_multiplier += multiplier;
        }   
    }
    col /= total_multiplier;
    // col /= ((kernel_size * 2.0) + 1.0) * ((kernel_size * 2.0) + 1.0);
    
    return col;
}