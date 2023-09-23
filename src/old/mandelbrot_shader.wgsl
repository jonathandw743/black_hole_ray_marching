
// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
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
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0); // 2.
    return out;
}

// Fragment shader

struct ResolutionUniform {
    res: vec2<f32>,
}
@group(0) @binding(0) // 1.
var<uniform> resolution: ResolutionUniform;

struct AntiAliasingUniform {
    number: f32,
}
@group(1) @binding(1) // 1.
var<uniform> anti_aliasing: AntiAliasingUniform;

fn complex_sq(z: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn get_col(c: vec2<f32>) -> vec3<f32> {
    var z = vec2<f32>(0.0);
    var max_iterations = 50.0;
    let r = 20.0;
    let r_sq = r * r;
    for (var i = 0.0; i < max_iterations; i += 1.0) {
        z = complex_sq(z) + c;
        let dist_sq = dot(z, z);
        if dist_sq > r_sq {
            let dist = sqrt(dist_sq);
            i -= log2(log(dist) / log(r)) - 1.0;
            let m = sqrt(i / max_iterations);
            return vec3<f32>(sin(vec3<f32>(0.8, 0.7, 0.5) * m * 20.0) * 0.5 + 0.5);
        }
    }
    return vec3<f32>(0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let min_resolution_dimension: f32 = min(resolution.res.x, resolution.res.y);
    let c = 4.0 * (in.uv - 0.5) * resolution.res.xy / min_resolution_dimension;
    let pixel_width = 4.0 / min_resolution_dimension;
    let aan = anti_aliasing.number;
    let aaside = aan * 2.0 + 1.0;
    let aawidth = pixel_width / aaside;
    let aamaxdiff = aawidth * aan;
    let aacount = aaside * aaside;
    var col = vec3<f32>(0.0);
    for (var xi = 0.0; xi < aaside; xi += 1.0) {
        for (var yi = 0.0; yi < aaside; yi += 1.0) {
            let x = c.x + (xi * aawidth) - aamaxdiff;
            let y = c.y + (yi * aawidth) - aamaxdiff;
            col += get_col(vec2<f32>(x, y));
        }
    }
    col /= aacount;
    return vec4<f32>(col, 1.0);
}