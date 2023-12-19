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
var s_diffuse: sampler;
// @group(0) @binding(2)
// var<uniform> resolution: vec2<u32>;

// fn map_col_component(component: f32) -> f32 {
//     // return 1.0 - exp(-component);
//     return 1.0 - 1.0 / (component + 1.0);
// }

// fn map_col(col: vec3<f32>) -> vec3<f32> {
//     return vec3<f32>(map_col_component(col.x), map_col_component(col.y), map_col_component(col.z));
// }

fn keep_bright_comp(x: f32) -> f32 {
    if x < 1.0 {
        return 0.0;
    } else {
        return x;
    }
}

fn keep_bright_col(col: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(keep_bright_comp(col.x), keep_bright_comp(col.y), keep_bright_comp(col.z));
}

fn map_comp(x: f32) -> f32 {
    return 1.0 / (1.0 - x) - 1.0;
}

fn map_col(col: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(map_comp(col.x), map_comp(col.y), map_comp(col.z));
    // return col;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let col = textureLoad(t_diffuse, in.pixel_uv, 0).xyz;
    // let col = vec3<f32>(, 0.0);
    // let in_col = textureLoad(t_diffuse, vec2<u32>(in.uv * vec2<f32>(resolution)), 0).xyz;
    let in_col = textureSample(t_diffuse, s_diffuse, in.uv).xyz;
    let mapped_in_col = map_col(in_col);
    let col = (mapped_in_col);
    return vec4<f32>(col, 1.0);
}

fn tsw(t_diffuse: texture_2d<f32>, s_difuse: sampler, tex_coords: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}