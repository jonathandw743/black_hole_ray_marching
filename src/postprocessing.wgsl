
struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) position: vec3<f32>,
    // @location(1) camera_to_vertex: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // out.camera_to_vertex = model.position - camera.pos.xyz;
    out.clip_position = vec4<f32>(model.position, 1.0); // 2.
    // out.position = model.position;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn tsw(t_diffuse: texture_2d<f32>, s_difuse: sampler, tex_coords: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}

fn map_col_component(component: f32) -> f32 {
    // return 1.0 - exp(-component);
    return 1.0 - 1.0 / (component + 1.0);
}

fn map_col(col: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(map_col_component(col.x), map_col_component(col.y), map_col_component(col.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let inputcol = textureSample(t_diffuse, s_diffuse, in.clip_position.xy / 1000.0).xyz;
    let outputcol = inputcol * vec3<f32>(0.0, 0.0, 1.0);
    return vec4<f32>(outputcol, 1.0);
}