// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    pos: vec3<f32>,
};
@group(2) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    // @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    return out;
}

// Fragment shader

struct ResolutionUniform {
    res: vec2<f32>,
}
@group(0) @binding(0) // 1.
var<uniform> resolution_uniform: ResolutionUniform;

struct AntiAliasingUniform {
    number: f32,
}
@group(1) @binding(0) // 1.
var<uniform> anti_aliasing_uniform: AntiAliasingUniform;

struct AntiAliasing {
    number: f32,
    samples_along_side: f32,
    dist_between_samples: f32,
    farthest_sample_dist: f32,
    sample_count: f32,
}

fn create_anti_aliasing(pixel_width_uv: f32, anti_aliasing_number: f32) -> AntiAliasing {
    let number = anti_aliasing_number;
    let samples_along_side = number * 2.0 + 1.0;
    let dist_between_samples = pixel_width_uv / samples_along_side;
    let farthest_sample_dist = number * dist_between_samples;
    let sample_count = samples_along_side * samples_along_side;
    return AntiAliasing(
        number,
        samples_along_side,
        dist_between_samples,
        farthest_sample_dist,
        sample_count,
    );
}

const MAX_STEPS = 100;
const MAX_DIST = 100.0;
const MIN_DIST = 1e-3;
const EPSILON_VEC: vec2<f32> = vec2<f32>(1e-3, 0.0);

// const RAY_ORIGIN: vec3<f32> = vec3<f32>(0.3, 0.1, -3.0);

const LIGHT_DIR: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
const LIGHT_POS: vec3<f32> = vec3<f32>(0.0, 2.0, 0.0);

const LIGHT_OPTION: i32 = 0;

const SHADOWS_ENABLED: bool = true;

fn sdf_sphere(p: vec3<f32>, centre: vec3<f32>, r: f32) -> f32 {
    return length(centre - p) - r;
}

fn get_dist(p: vec3<f32>) -> f32 {
    let sd_sphere_1 = sdf_sphere(p, vec3<f32>(0.0), 0.5);
    let sd_sphere_2 = sdf_sphere(p, vec3<f32>(0.0, -1.0, 0.0), 1.0);
    return sd_sphere_2;
    // return min(sd_sphere_1, sd_sphere_2);
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let dist = get_dist(p);

    let normal = normalize(dist - vec3<f32>(
        get_dist(p - EPSILON_VEC.xyy),
        get_dist(p - EPSILON_VEC.yxy),
        get_dist(p - EPSILON_VEC.yyx)
    ));
    
    return normal;
}

struct Distance {
    dist: f32,
    is_infinte: bool,
}

fn dist_less_than(d0: Distance, d1: Distance) -> bool {
    if d0.is_infinte {
        return false;
    }
    if d1.is_infinte {
        return true;
    }
    if d0.dist < d1.dist {
        return true;
    }
    return false;
}

fn ray_march(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> Distance{
    var dist_travelled = 0.0;
    for (var i = 0; i < MAX_STEPS; i++) {
        let p = ray_origin + dist_travelled * ray_direction;
        let surface_dist = get_dist(p);
        dist_travelled += surface_dist;
        // check for infinite distance
        if dist_travelled > MAX_DIST {
            break;
        }
        // check for hit surface
        if surface_dist < MIN_DIST {
            return Distance(dist_travelled, false);
        }
    }
    // this represents an infinite distance
    return Distance(0.0, true);
}

fn is_in_shadow(p: vec3<f32>, normal: vec3<f32>, light_dir: vec3<f32>, light_dist_sq: Distance) -> bool {
    if !SHADOWS_ENABLED {
        return false;
    }
    let uninterrupted_light_dist = ray_march(p + 1.0 * MIN_DIST * normal, light_dir);
    if uninterrupted_light_dist.is_infinte {
        return false;
    }
    let uninterrupted_light_dist_sq = Distance(
        uninterrupted_light_dist.dist *
        uninterrupted_light_dist.dist,
        false,
    );
    return dist_less_than(uninterrupted_light_dist_sq, light_dist_sq);
}

fn get_directional_light(normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    return dot(normal, light_dir);
}

fn get_light(p: vec3<f32>) -> f32 {
    let normal = get_normal(p);

    var light_dir: vec3<f32>;
    var light_dist_sq: Distance;

    switch LIGHT_OPTION {
        case 0 {
            light_dir = LIGHT_DIR;
            light_dist_sq = Distance(0.0, true);
        }
        case 1 {
            let to_light = LIGHT_POS - p;

            light_dir = normalize(to_light);
            light_dist_sq = Distance(dot(to_light, to_light), false);
        }
        default {
            light_dir = LIGHT_DIR;
            light_dist_sq = Distance(0.0, true);
        }
    }

    if is_in_shadow(p, normal, light_dir, light_dist_sq) {
        return 0.0;
    } else {
        return get_directional_light(normal, light_dir);
    }
}

fn get_col(uv: vec2<f32>) -> vec3<f32> {
    let ray_direction = vec3(uv.xy, 1.0);
    let dist = ray_march(camera.pos, ray_direction);
    if dist.is_infinte {
        return vec3<f32>(0.0);
    } else {
        let p = camera.pos + dist.dist * ray_direction;
        let col = vec3<f32>(get_light(p));
        return col;
    }
}

fn get_col_with_anti_aliasing(uv: vec2<f32>, anti_aliasing: AntiAliasing) -> vec3<f32> {
    var col = vec3<f32>(0.0);
    for (var xi = 0.0; xi < anti_aliasing.samples_along_side; xi += 1.0) {
        for (var yi = 0.0; yi < anti_aliasing.samples_along_side; yi += 1.0) {
            let x: f32 = uv.x + (xi * anti_aliasing.dist_between_samples) - anti_aliasing.farthest_sample_dist;
            let y: f32 = uv.y + (yi * anti_aliasing.dist_between_samples) - anti_aliasing.farthest_sample_dist;
            col += get_col(vec2<f32>(x, y));
        }
    }
    col /= anti_aliasing.sample_count;
    return col;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let min_dimension_pixels = min(resolution_uniform.res.x, resolution_uniform.res.y);
    let min_dimension_uv = 1.0;
    let uv = min_dimension_uv * (in.uv.xy - 0.5) * resolution_uniform.res.xy / min_dimension_pixels;
    let pixel_width_uv = min_dimension_uv / min_dimension_pixels;

    if anti_aliasing_uniform.number == 0.0 {
        let col = get_col(uv);
        return vec4<f32>(col, 1.0);
    } else {
        let anti_aliasing = create_anti_aliasing(pixel_width_uv, anti_aliasing_uniform.number);
        let col = get_col_with_anti_aliasing(uv, anti_aliasing);
        return vec4<f32>(col, 1.0);
    }
}