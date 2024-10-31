// for a system where:
// c = 1
// GM = 1
// .:
// r_s = 2
// V = -1/r

struct Camera {
    // 0B
    pos: vec3<f32>,
    // has to be vec4f for correct array stride (16B) 
    screen_space_screen_triangle: array<vec4<f32>, 3>,
    pos_to_world_space_screen_triangle: array<vec4<f32>, 3>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(1) camera_to_vertex: vec3<f32>,
}

@vertex
fn vs_main(
@builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.screen_space_screen_triangle[vertex_index];
    out.clip_position.z = 0.0;
    out.clip_position.w = 1.0;
    out.camera_to_vertex = camera.pos_to_world_space_screen_triangle[vertex_index].xyz;
    return out;
}

struct BlackHole {
    // 0B
    pos: vec3f,
    rs: f32,
    accretion_disk_size: f32,
    // 16B
}

struct BlackHolesUniform {
    black_holes: array<BlackHole, MAX_BLACK_HOLE_COUNT>,
    count: u32,
}

struct Photon {
    ro: vec3<f32>,
    rd: vec3<f32>,
}

const MAX_BLACK_HOLE_COUNT = 10u;

struct Uniforms {
    // 0B
    dist_to_surfaces_mult: f32,
    dist_to_singularity_squared_mult: f32,
    blackout_eh: u32,
    min_dist: f32,
    max_dist: f32,
    distortion_power: f32,
    debug_colours: u32,
    blackout_requires_ray_towards_black_hole: u32,
    photon_sphere: u32,
    // 40B
    // padding?
}

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@group(0) @binding(2)
var<uniform> black_holes_uniform: BlackHolesUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

const EPSILON_VEC: vec2<f32> = vec2<f32>(1e-3, 0.0);
const TWO_PI = 6.28318530718;
const ONE_PI = 3.14159265359;
const HALF_PI = 1.57079632679;
const MAX_ITERATIONS = 1000;
const PLANE_THICKNESS = 0.0;
const INNERMOST_STABLE_ORBIT = 3.0;

fn u32_to_bool(n: u32) -> bool {
    return n != 0u;
}

fn sdf_sphere(p: vec3<f32>, centre: vec3<f32>, r: f32) -> f32 {
    return length(p - centre) - r;
}

fn sdf_plane(p: vec3<f32>, y: f32) -> f32 {
    return abs(p.y - y) - PLANE_THICKNESS;
}

fn sdf_cylinder(p: vec3<f32>, pos: vec2<f32>, radius: f32) -> f32 {
    return length(p.xz - pos) - radius;
}

fn sdf_accretion_disk(p: vec3<f32>, centre: vec3<f32>, big_r: f32, little_r: f32) -> f32 {
    return max(max(sdf_cylinder(p, centre.xz, big_r), -sdf_cylinder(p, centre.xz, little_r)), sdf_plane(p, centre.y));
}

fn sdf_markers(p: vec3f) -> f32 {
    let sd_sphere_1 = sdf_sphere(p, vec3<f32>(0.0, 10.0, -10.0), 0.5);
    let sd_sphere_2 = sdf_sphere(p, vec3<f32>(0.0, -10.0, -10.0), 0.5);
    let sd_sphere_3 = sdf_sphere(p, vec3<f32>(10.0, 0.0, -10.0), 0.5);
    let sd_sphere_4 = sdf_sphere(p, vec3<f32>(-10.0, 0.0, -10.0), 0.5);
    return min(sd_sphere_1, min(sd_sphere_2, min(sd_sphere_3, sd_sphere_4)));
}

fn sdf(p: vec3<f32>) -> f32 {
    var sd = sdf_markers(p);
    for (var i = 0u; i < black_holes_uniform.count; i++) {
        let sd_accretion_disk = sdf_accretion_disk(
            p, vec3<f32>(black_holes_uniform.black_holes[i].pos),
            black_holes_uniform.black_holes[i].accretion_disk_size * black_holes_uniform.black_holes[i].rs,
            INNERMOST_STABLE_ORBIT * black_holes_uniform.black_holes[i].rs
        );
        sd = min(sd, sd_accretion_disk);
    }
    return sd;
}

fn rd_derivative(ro: vec3<f32>, h2s: array<f32, MAX_BLACK_HOLE_COUNT>) -> vec3<f32> {
    var res = vec3f(0.0);
    var varh2s = h2s;
    for (var i = 0u; i < black_holes_uniform.count; i++) {
        let relative_pos = ro - black_holes_uniform.black_holes[i].pos;
        res += uniforms.distortion_power * black_holes_uniform.black_holes[i].rs * -1.5 * varh2s[i] * relative_pos / pow(dot(relative_pos, relative_pos), 2.5);
    }
    return res;
}

fn get_delta_photon_rk4(photon: Photon, delta_time: f32, h2s: array<f32, MAX_BLACK_HOLE_COUNT>) -> Photon {
    let ro_k1 = delta_time * photon.rd;
    let rd_k1 = delta_time * rd_derivative(photon.ro, h2s);
    
    let ro_k2 = delta_time * (photon.rd + 0.5 * rd_k1);
    let rd_k2 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k1, h2s);
    
    let ro_k3 = delta_time * (photon.rd + 0.5 * rd_k2);
    let rd_k3 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k2, h2s);
    
    let ro_k4 = delta_time * (photon.rd + rd_k3);
    let rd_k4 = delta_time * rd_derivative(photon.ro + ro_k3, h2s);

    let delta_ro = (ro_k1 + 2.0 * ro_k2 + 2.0 * ro_k3 + ro_k4) / 6.0;
    let delta_rd = (rd_k1 + 2.0 * rd_k2 + 2.0 * rd_k3 + rd_k4) / 6.0;

    return Photon(delta_ro, delta_rd);
}

fn tsw(t: texture_2d<f32>, s: sampler, p: vec2f) -> vec4f {
    return textureSample(t, s, p);
}

fn map_bg_col(bg_col: vec3f) -> vec3f {
    return bg_col;
    // var res = bg_col;
    // res.y = pow(res.y, 1.5);
    // res.z = pow(res.z, 1.5);
    // return res;
}

fn get_col(initial_photon: Photon) -> vec3<f32> {
    var photon = Photon(initial_photon.ro, initial_photon.rd);

    var h2s: array<f32, MAX_BLACK_HOLE_COUNT>;
    for (var i = 0u; i < black_holes_uniform.count; i++) {
        let initial_ro_rd_cross = cross(photon.ro - black_holes_uniform.black_holes[i].pos, photon.rd);
        h2s[i] = dot(initial_ro_rd_cross, initial_ro_rd_cross);
    }

    var distance_travelled = 0.0;

    // var has_been_outside_ehs: array<bool, MAX_BLACK_HOLE_COUNT>;
    // for (var i = 0u; i < MAX_BLACK_HOLE_COUNT; i++) {
    //     if (black_holes_uniform.black_holes[i].rs == 0.0) { break; }
    //     has_been_outside_ehs[i] = false;
    // }

    for (var i = 0; i < MAX_ITERATIONS; i++) {
        // the photon should approach the singularity
        // given the desired distance calculation
        // just like the ray approaches a surface in raymarching
        // there also be some distance to the singularity that will cause black
        var dists_to_singularities: array<f32, MAX_BLACK_HOLE_COUNT>;
        for (var i = 0u; i < black_holes_uniform.count; i++) {
            dists_to_singularities[i] = length(photon.ro - black_holes_uniform.black_holes[i].pos);
        }

        var dist_to_surfaces = sdf(photon.ro);
        if dist_to_surfaces < uniforms.min_dist {
            return vec3<f32>(1.0);
        }

        // photon is a small sphere at the back of the black hole
        // distance of 1.5 * r_s away
        // we say that if a photon hits this sphere, it goes into temporary orbit around the black hole
        // https://upload.wikimedia.org/wikipedia/commons/2/27/Black_Hole_Shadow.gif

        if u32_to_bool(uniforms.photon_sphere) {
            var photon_sphere_dist = uniforms.max_dist;
            for (var i = 0u; i < black_holes_uniform.count; i++) {
                photon_sphere_dist = min(
                    photon_sphere_dist,
                    sdf_sphere(photon.ro, black_holes_uniform.black_holes[i].pos - normalize(initial_photon.ro) * 1.5 * black_holes_uniform.black_holes[i].rs, 0.1)
                );
            }
            if photon_sphere_dist < uniforms.min_dist {
                return vec3<f32>(1.0, 1.0, 0.0);
            }
            dist_to_surfaces = min(dist_to_surfaces, photon_sphere_dist);
        }

        // the photon should be able to travel further if it is far away from the black hole
        // k * distance to singularity
        // so each step, the maximum distance the photon can travel is about half the distance to the event horizon
        // also, when travelling away from the black hole,
        // the distance away from the black hole should grow exponentially
        // this means max view distance can be increased massively
        // then apply the ray marching distance
        // 0.9 multiplier just to account for any error due to the curvature of the ray
        var delta_time = dist_to_surfaces * uniforms.dist_to_surfaces_mult;
        for (var i = 0u; i < black_holes_uniform.count; i++) {
            delta_time = min(delta_time, uniforms.dist_to_singularity_squared_mult * dists_to_singularities[i] * dists_to_singularities[i]);
        }

        // how the photon should move given the desired distance and the current state of the photonn
        // let delta_photon = get_delta_photon_rk4(photon, delta_time, h2s);
        let ro_k1 = delta_time * photon.rd;
        let rd_k1 = delta_time * rd_derivative(photon.ro, h2s);
        
        let ro_k2 = delta_time * (photon.rd + 0.5 * rd_k1);
        let rd_k2 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k1, h2s);
        
        let ro_k3 = delta_time * (photon.rd + 0.5 * rd_k2);
        let rd_k3 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k2, h2s);
        
        let ro_k4 = delta_time * (photon.rd + rd_k3);
        let rd_k4 = delta_time * rd_derivative(photon.ro + ro_k3, h2s);

        let delta_ro = (ro_k1 + 2.0 * ro_k2 + 2.0 * ro_k3 + ro_k4) / 6.0;
        let delta_rd = (rd_k1 + 2.0 * rd_k2 + 2.0 * rd_k3 + rd_k4) / 6.0;

        let delta_photon = Photon(delta_ro, delta_rd);

        photon.ro += delta_photon.ro;
        // photon.rd won't be a unit vector at all points in the loop
        // so there's no guarantee that the distance travelled along the light path
        // equals dd
        // but it is a good approximation
        // and its too expensive to actually ensure this
        // but its a good enough appoximation
        photon.rd += delta_photon.rd;

        if u32_to_bool(uniforms.blackout_eh) {
            for (var i = 0u; i < black_holes_uniform.count; i++) {
                // if (black_holes_uniform.black_holes[i].rs == 0.0) { break; }
                if dists_to_singularities[i] < black_holes_uniform.black_holes[i].rs && (
                    !u32_to_bool(uniforms.blackout_requires_ray_towards_black_hole) || 
                    dot(photon.ro, photon.rd) < 0.0
                ) {
                    return vec3f(0.0);
                }
            }
        }

        distance_travelled += delta_time;
        if distance_travelled > uniforms.max_dist {
            if u32_to_bool(uniforms.debug_colours) {
                return vec3f(1.0, 0.0, 0.0);
            }
            break;
        }
    }
    if u32_to_bool(uniforms.debug_colours) {
        return vec3f(0.0, 1.0, 0.0);
    }
    // any unit vector
    let normalized_final_rd = normalize(photon.rd);
    // range -PI to +PI
    let azimuthal_angle = atan2(normalized_final_rd.z, normalized_final_rd.x);
    // range 0 to 1
    let x = (azimuthal_angle + ONE_PI) / TWO_PI;
    // range 0 to 1
    let y = (normalized_final_rd.y + 1.0) * 0.5;

    // 1 - y because in texture coords, +y is down
    let bg_col = textureSampleLevel(t_diffuse, s_diffuse, vec2<f32>(x, 1.0 - y), 0.0).xyz;
    let mapped_bg_col = map_bg_col(bg_col);
    return mapped_bg_col;
}

struct FragmentOutput {
    @location(0) col: vec4<f32>,
    @location(1) blackout_col: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let ray_dir = normalize(in.camera_to_vertex);
    let photon = Photon(camera.pos.xyz, ray_dir);
    let col = get_col(photon);
    var blackout_col = col;
    if dot(col, col) < 1.0 {
        blackout_col = vec3f(0.0);
    }
    return FragmentOutput(vec4<f32>(col, 1.0), vec4<f32>(blackout_col, 1.0));
}