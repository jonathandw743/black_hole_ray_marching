struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.uv = model.uv;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var<uniform> resolution: vec2<u32>;

const BLUR_AMOUNT: f32 = 8.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_pos = in.uv * vec2<f32>(resolution) / BLUR_AMOUNT;
    let fractpp = fract(pixel_pos);
    let floorpp = floor(pixel_pos) * BLUR_AMOUNT;
    let nextpp = (floorpp + vec2<f32>(1.0)) * BLUR_AMOUNT;
    let floorcol = textureLoad(t_diffuse, floorpp, 0);
    let nextcolx = textureLoad(t_diffuse, vec2<f32>(nextpp.x, floorpp.x), 0);
    let nextcoly = textureLoad(t_diffuse, vec2<f32>(floorpp.x, nextpp.y), 0);
    let nextcol = textureLoad(t_diffuse, nextpp, 0);
    let colx1 = mix(floorcol, nextcolx, fractpp.x);
    let colx2 = mix(nextcoly, nextcol, fractpp.x);
    let col = mix(colx1, colx2, fractpp.y);
    return vec4<f32>(col, 1.0);
}