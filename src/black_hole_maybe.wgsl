// for a systems where:
// c = 1
// GM = 1
// .:
// r_s = 2
// V = -1/r

// Vertex shader
struct Camera {
    // 0 bytes
    pos: vec3<f32>,
    // view_proj: mat4x4<f32>,
    // inverse_view_proj: mat4x4<f32>,
 // has to be vec4 for correct array stride 
  screen_space_screen_triangle: array<vec4<f32>, 3>,
  pos_to_world_space_screen_triangle: array<vec4<f32>, 3>,
}
@group(0) @binding(0) // 1.
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
  @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
   @invariant @builtin(position) clip_position: vec4<f32>,
    @location(1) camera_to_vertex: vec3<f32>,
}

// var<private> positions: array<vec2f, 3> = array<vec2f, 3>(
//     vec2f(3.0, 1.0),
//     vec2f(-1.0, 1.0),
//     vec2f(-1.0, -3.0),
// );

@vertex
fn vs_main(
@builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
  // out.position = vec4f(positions[vertex_index], 0.0, 1.0);
  // var world_pos_homogeneous = camera.inverse_view_proj * out.position;
  // world_pos_homogeneous /= world_pos_homogeneous.w;
    // out.camera_to_vertex = world_pos_homogeneous.xyz - camera.pos.xyz;
    // out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
  
  out.clip_position = camera.screen_space_screen_triangle[vertex_index];
  out.clip_position.z = 0.0;
  out.clip_position.w = 1.0;
  // out.clip_position = vec4f(0.0);
  out.camera_to_vertex = camera.pos_to_world_space_screen_triangle[vertex_index].xyz;
  // out.camera_to_vertex = vec3f(0.0);
    return out;
}

// Fragment shader
struct Uniforms {
    // 0 bytes
    RS: f32,
    DELTA_TIME_MULT: f32,
    BG_BRIGHTNESS: f32,
    BLACKOUT_EH: u32,
    MAX_DIST: f32,
    DISTORTION_POWER: f32,
    // 24 bytes
    _padding: vec2<u32>,
    // 32 bytes (16x2)
}

@group(0) @binding(1)
var<uniform> u: Uniforms;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

const BH_POS: vec3f = vec3f(0.0);
const MIN_DIST = 0.001;
const EPSILON_VEC: vec2<f32> = vec2<f32>(1e-3, 0.0);
const TWO_PI = 6.28318530718;
const ONE_PI = 3.14159265359;
const HALF_PI = 1.57079632679;
const MAX_ITERATIONS = 1000;

fn u32_to_bool(n: u32) -> bool {
    return n != 0u;
}

fn sdf_sphere(p: vec3<f32>, centre: vec3<f32>, r: f32) -> f32 {
    return length(centre - p) - r;
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

fn sdf_markers(p: vec3f) -> f32 {
    let sd_sphere_1 = sdf_sphere(p, vec3<f32>(0.0, 10.0, -10.0), 0.5);
    let sd_sphere_2 = sdf_sphere(p, vec3<f32>(0.0, -10.0, -10.0), 0.5);
    let sd_sphere_3 = sdf_sphere(p, vec3<f32>(10.0, 0.0, -10.0), 0.5);
    let sd_sphere_4 = sdf_sphere(p, vec3<f32>(-10.0, 0.0, -10.0), 0.5);
    // let sd_sphere_1 = sdf_sphere(p, vec3<f32>(0.0, 5.0, -10.0), 0.5);
    // let sd_sphere_2 = sdf_sphere(p, vec3<f32>(0.0, -5.0, -10.0), 0.5);
    // let sd_sphere_3 = sdf_sphere(p, vec3<f32>(5.0, 0.0, -10.0), 0.5);
    // let sd_sphere_4 = sdf_sphere(p, vec3<f32>(-5.0, 0.0, -10.0), 0.5);
  return min(sd_sphere_1, min(sd_sphere_2, min(sd_sphere_3, sd_sphere_4)));
}

fn sdf(p: vec3<f32>) -> f32 {
    let sd_accretion_disk = sdf_accretion_disk(p, vec3<f32>(0.0), 6.0 * u.RS, 3.0 * u.RS);
    let sd_markers = sdf_markers(p);
  return min(sd_accretion_disk, sd_markers);
}

fn rd_derivative(ro: vec3<f32>, h2: f32) -> vec3<f32> {
    return u.DISTORTION_POWER * u.RS * -1.5 * h2 * ro / pow(dot(ro, ro), 2.5);
}

struct Photon {
    ro: vec3<f32>,
    rd: vec3<f32>,
}

fn get_delta_photon_rk4(photon: Photon, delta_time: f32, h2: f32) -> Photon {
    let ro_k1 = delta_time * photon.rd;
    let rd_k1 = delta_time * rd_derivative(photon.ro, h2);
    
    let ro_k2 = delta_time * (photon.rd + 0.5 * rd_k1);
    let rd_k2 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k1, h2);
    
    let ro_k3 = delta_time * (photon.rd + 0.5 * rd_k2);
    let rd_k3 = delta_time * rd_derivative(photon.ro + 0.5 * ro_k2, h2);
    
    let ro_k4 = delta_time * (photon.rd + rd_k3);
    let rd_k4 = delta_time * rd_derivative(photon.ro + ro_k3, h2);

    let delta_ro = (ro_k1 + 2.0 * ro_k2 + 2.0 * ro_k3 + ro_k4) / 6.0;
    let delta_rd = (rd_k1 + 2.0 * rd_k2 + 2.0 * rd_k3 + rd_k4) / 6.0;

    return Photon(delta_ro, delta_rd);
}
  
    fn rotate_vector(vector: vec3f, unit_axis: vec3f, angle: f32) -> vec3f {
    let cos_theta = cos(angle);
 let    sin_theta = sin(angle);
  // Calculate the rotation matrix components
    let ux = unit_axis.x;
    let uy = unit_axis.y;
    let uz = unit_axis.z;
    let one_minus_cos = 1.0 - cos_theta;

    // Apply the rotation formula
    let x = vector.x * (cos_theta + ux * ux * one_minus_cos) + 
            vector.y * (ux * uy * one_minus_cos - uz * sin_theta) + 
            vector.z * (ux * uz * one_minus_cos + uy * sin_theta);
    let y = vector.x * (uy * ux * one_minus_cos + uz * sin_theta) + 
            vector.y * (cos_theta + uy * uy * one_minus_cos) + 
            vector.z * (uy * uz * one_minus_cos - ux * sin_theta);
    let z = vector.x * (uz * ux * one_minus_cos - uy * sin_theta) + 
            vector.y * (uz * uy * one_minus_cos + ux * sin_theta) + 
            vector.z * (cos_theta + uz * uz * one_minus_cos);

    return vec3<f32>(x, y, z);
  }

fn tsw(t: texture_2d<f32>, s: sampler, p: vec2f) -> vec4f {
  return textureSample(t, s, p);
}

fn linearTextureSampleTest(t: texture_2d<f32>, s: sampler, p: vec2f) -> vec4f {
  let dim = vec2f(textureDimensions(t));
  let inv_dim = 1.0 / dim;
  let p_pixel_space = p * dim;
  let p_floor = floor(p_pixel_space) * inv_dim;
  // let p_ceil = ceil(p_pixep_pixel_space) * inv_dim;
  let p_ceil = p_floor + inv_dim;
  let a = textureSampleLevel(t, s, p_floor, 0.0);
  let b = textureSampleLevel(t, s, vec2f(p_ceil.x, p_floor.x), 0.0);
  let c = textureSampleLevel(t, s, vec2f(p_floor.x, p_ceil.x), 0.0);
  let d = textureSampleLevel(t, s, p_ceil, 0.0);
  let p_fract = p - p_floor;
  let ab = a * p_fract.x + b * (1.0 - p_fract.x);
  let cd = c * p_fract.x + d * (1.0 - p_fract.x);
  return ab * p_fract.y + cd * (1.0 - p_fract.y);
}
/*
fn ray_march_photon(photon: Photon) -> vec3f {
    let initial_ro_rd_cross = cross(photon.ro, photon.rd);
    let h2 = dot(initial_ro_rd_cross, initial_ro_rd_cross);
    var distance_travelled = 0.0;
  
    for (var i = 0; i < MAX_ITERATIONS; i++) {
        // the photon should approach the event horizon
        // given the desired distance calculation
        // just like the ray approaches a surface in raymarching
        let dist_to_eh = sdf_sphere(photon.ro, BH_POS, u.RS);
        if dist_to_eh < MIN_DIST {
            return BH_POS;
        }
    
        let dist_to_surfaces = sdf(photon.ro);
        if dist_to_surfaces < MIN_DIST {
            return photon.ro;
        }

        // photon is a small sphere at the back of the black hole
        // distance of 1.5 * r_s away
        // we say that if a photon hits this sphere, it goes into temporary orbit around the black hole
        // https://upload.wikimedia.org/wikipedia/commons/2/27/Black_Hole_Shadow.gif
        let photon_sphere_dist = sdf_sphere(photon.ro, BH_POS - normalize(initial_photon.ro) * 1.5 * u.RS, 0.1);
        if photon_sphere_dist < MIN_DIST {
            return BH_POS ;
        }

        let dist = min(dist_to_surfaces, photon_sphere_dist);

        // the photon should be able to travel further if it is far away from the black hole
        // k * distance to event horizon
        // so each step, the maximum distance the photon can travel is about half the distance to the event horizon
        // also, when travelling away from the black hole,
        // the distance away from the black hole should grow exponentially
        // this means max view distance can be increased massively
        var dd = u.DELTA_TIME_MULT * dist_to_eh;
        // then apply the ray marching distance
        // 0.9 multiplier just to account for any error due to the curvature of the ray
        dd = min(dist * 0.9, dd);

        // how the photon should move given the desired distance and the current state of the photonn
        let delta_photon = get_delta_photon_rk4(photon, dd, h2);

        photon.ro += delta_photon.ro;
        // photon.rd won't be a unit vector at all points in the loop
        // so there's no guarantee that the distance travelled along the light path
        // equals dd
        // but it is a good approximation
        // and its too expensive to actually ensure this
        // but its a good enough appoximation
        photon.rd += delta_photon.rd;

        distance_travelled += dd;
        if distance_travelled > u.MAX_DIST {
            break;
        }
    }
}
*/
fn get_col(initial_photon: Photon) -> vec3<f32> {
    var photon = Photon(initial_photon.ro, initial_photon.rd); 

    let initial_ro_rd_cross = cross(photon.ro, photon.rd);
    let h2 = dot(initial_ro_rd_cross, initial_ro_rd_cross);
    var distance_travelled = 0.0;
    var has_been_outside_eh = false;
    for (var i = 0; i < MAX_ITERATIONS; i++) {
        // the photon should approach the singularity
        // given the desired distance calculation
        // just like the ray approaches a surface in raymarching
        // there also be some distance to the singularity that will cause black
        let dist_to_singularity = length(photon.ro);
        if u32_to_bool(u.BLACKOUT_EH) {
            if dist_to_singularity < 1.0 {
                if dot(photon.rd, photon.ro) < 0.0 {
                    return vec3<f32>(0.0);
                }
            }
            if dist_to_singularity > 1.0 {
                has_been_outside_eh = true;
            } else if has_been_outside_eh {
                return vec3<f32>(0.0);
            }
        }

        let dist_to_surfaces = sdf(photon.ro);
        if dist_to_surfaces < MIN_DIST {
            return vec3<f32>(1.0);
        }

        // photon is a small sphere at the back of the black hole
        // distance of 1.5 * r_s away
        // we say that if a photon hits this sphere, it goes into temporary orbit around the black hole
        // https://upload.wikimedia.org/wikipedia/commons/2/27/Black_Hole_Shadow.gif
        let photon_sphere_dist = sdf_sphere(photon.ro, -normalize(initial_photon.ro) * 1.5 * u.RS, 0.075);
        //if photon_sphere_dist < MIN_DIST {
        //    return vec3<f32>(1.0, 1.0, 0.0);
        //}

        let dist = min(dist_to_surfaces, photon_sphere_dist);

        // the photon should be able to travel further if it is far away from the black hole
        // k * distance to event horizon
        // so each step, the maximum distance the photon can travel is about half the distance to the event horizon
        // also, when travelling away from the black hole,
        // the distance away from the black hole should grow exponentially
        // this means max view distance can be increased massively
        var dd = u.DELTA_TIME_MULT * dist_to_singularity;
        // then apply the ray marching distance
        // 0.9 multiplier just to account for any error due to the curvature of the ray
        dd = min(dist * 0.9, dd);

        // how the photon should move given the desired distance and the current state of the photonn
        let delta_photon = get_delta_photon_rk4(photon, dd, h2);

        photon.ro += delta_photon.ro;
        // photon.rd won't be a unit vector at all points in the loop
        // so there's no guarantee that the distance travelled along the light path
        // equals dd
        // but it is a good approximation
        // and its too expensive to actually ensure this
        // but its a good enough appoximation
        photon.rd += delta_photon.rd;

        distance_travelled += dd;
        if distance_travelled > u.MAX_DIST {
            break;
        }
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
    // let col = tsw(t_diffuse, s_diffuse, vec2<f32>(x, 1.0 - y)).xyz;
    // let col = textureSampleLevel(t_diffuse, s_diffuse, vec2<f32>(floor(x), floor(1.0 - y)), 0.0).xyz;
    let col = linearTextureSampleTest(t_diffuse, s_diffuse, vec2<f32>(x, 1.0 - y)).xyz;
    return col;
}

fn map_col_component_infinity_to_one(component: f32) -> f32 {
    return 1.0 - 1.0 / (component + 1.0);
}

fn map_col_infinity_to_one(col: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(map_col_component_infinity_to_one(col.x), map_col_component_infinity_to_one(col.y), map_col_component_infinity_to_one(col.z));
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
