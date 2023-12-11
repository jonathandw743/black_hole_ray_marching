// Vertex shader
struct Camera {
    // 0 bytes
    pos: vec3<f32>,
    view_proj: mat4x4<f32>,
    // 76 bytes
    // _padding: u32,
    // 80 bytes (16x5)
}
@group(0) @binding(0) // 1.
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) camera_to_vertex: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.camera_to_vertex = model.position - camera.pos.xyz;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    out.position = model.position;
    return out;
}

// Fragment shader
struct Uniforms {
    // 0 bytes
    RS: f32,
    MAX_DELTA_TIME: f32,
    BG_BRIGHTNESS: f32,
    BLACKOUT_EH: u32,
    MAX_DIST: f32,
    DISTORTION_POWER: f32,
    // 24 bytes
    // _padding: vec2<u32>,
    // 32 bytes (16x2)
}

@group(0) @binding(1)
var<uniform> u: Uniforms;

const MIN_DIST = 0.001;
const EPSILON_VEC: vec2<f32> = vec2<f32>(1e-3, 0.0);
const TWO_PI = 6.28318530718;
const ONE_PI = 3.14159265359;
const HALF_PI = 1.57079632679;
const MAX_ITERATIONS = 1000;

fn u32_to_bool(n: u32) -> bool {
    return n != 0u;
}

fn sdf_plane(p: vec3<f32>, y: f32) -> f32 {
    return abs(p.y - y) - 0.02;
}

fn sdf_cylinder(p: vec3<f32>, pos: vec2<f32>, radius: f32) -> f32 {
    return length(p.xz - pos) - radius;
}

fn sdf_accretion_disk(p: vec3<f32>, centre: vec3<f32>, big_r: f32, little_r: f32) -> f32 {
    return max(max(sdf_cylinder(p, centre.xz, big_r), -sdf_cylinder(p, centre.xz, little_r)), sdf_plane(p, centre.y));
}

fn get_dist(p: vec3<f32>) -> f32 {
    // let sd_sphere_bh = sdf_sphere(p, vec3<f32>(0.0, 0.0, 0.0), 0.8);

    let sd_accretion_disk = sdf_accretion_disk(p, vec3<f32>(0.0), 5.0 * u.RS, 3.0 * u.RS);

    // return min(sd_sphere_bh, sd_accretion_disk);
    return sd_accretion_disk;
}

fn rd_derivative(ro: vec3<f32>, h2: f32) -> vec3<f32> {
    // return vec3<f32>(0.0);

    return u.DISTORTION_POWER * -1.5 * h2 * ro / pow(dot(ro, ro), 2.5);
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

fn tsw(t_diffuse: texture_2d<f32>, s_difuse: sampler, tex_coords: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}

fn get_col(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec3<f32> {
    // let BLACK_HOLE: BlackHole = BlackHole(vec3<f32>(0.0, 0.0, 0.0), BLACK_HOLE_MASS);
    
    // var photon = Photon(ray_origin, ray_dir);
    var ro = ray_origin;
    var rd = ray_dir;
    let ro_rd_cross = cross(ro, rd);
    let h2 = dot(ro_rd_cross, ro_rd_cross);

    var distance_travelled = 0.0;
    // for (var i = 0; i < (2 * )i32(MAX_DIST / MAX_DELTA_TIME); i++) {
    for (var i = 0; i < MAX_ITERATIONS; i++) {

        let dist_to_bh_sq = dot(ro, ro);
        if u32_to_bool(u.BLACKOUT_EH) && dist_to_bh_sq < 1.0 {
            return vec3<f32>(0.0);
        }

        let dist_0 = get_dist(ro);
        if dist_0 < MIN_DIST {
            return vec3<f32>(1.0);
        }

        let ps_dist = sdf_sphere(ro, -normalize(ray_origin) * 1.5, 0.075);
        if ps_dist < MIN_DIST {
            return vec3<f32>(1.0, 1.0, 0.0);
        }

        let dist = min(dist_0, ps_dist);
        
        // can interpret dist as a time due to c = 1
        var delta_time = min(u.MAX_DELTA_TIME, dist);

        let ro_k1 = delta_time * rd;
        let rd_k1 = delta_time * rd_derivative(ro, h2);
        
        let ro_k2 = delta_time * (rd + 0.5 * rd_k1);
        let rd_k2 = delta_time * rd_derivative(ro + 0.5 * ro_k1, h2);
        
        let ro_k3 = delta_time * (rd + 0.5 * rd_k2);
        let rd_k3 = delta_time * rd_derivative(ro + 0.5 * ro_k2, h2);
        
        let ro_k4 = delta_time * (rd + rd_k3);
        let rd_k4 = delta_time * rd_derivative(ro + ro_k3, h2);

        let delta_ro = (ro_k1 + 2.0 * ro_k2 + 2.0 * ro_k3 + ro_k4) / 6.0;
        let delta_rd = (rd_k1 + 2.0 * rd_k2 + 2.0 * rd_k3 + rd_k4) / 6.0;

        ro += delta_ro;
        rd += delta_rd;

        distance_travelled += delta_time;
        if distance_travelled > u.MAX_DIST {
            // return vec3<f32>(0.0, 1.0, 1.0);
            break;
        }        
    }
    // let 
    let x = (atan2(rd.z, rd.x) + TWO_PI * 0.5) / TWO_PI;
    let y = (-rd.y + 1.0) * 0.5;
    let col = tsw(t_diffuse, s_diffuse, vec2<f32>(x, y)).xyz;
    return col;
    // return rd * BG_BRIGHTNESS;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_dir = normalize(in.camera_to_vertex);
    let col = get_col(camera.pos.xyz, ray_dir);
    return vec4<f32>(col, 1.0);
}