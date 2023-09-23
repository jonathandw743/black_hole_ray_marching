// Vertex shader
@group(0) @binding(0) // 1.
var<uniform> camera_pos: vec3<f32>;

@group(0) @binding(1) // 1.
var<uniform> camera_view_proj: mat4x4<f32>;

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
    out.camera_to_vertex = model.position - camera_pos;
    out.clip_position = camera_view_proj * vec4<f32>(model.position, 1.0); // 2.
    out.position = model.position;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var<uniform> RS: f32;
@group(1) @binding(1)
var<uniform> MAX_DELTA_TIME: f32;
@group(1) @binding(2)
var<uniform> BG_BRIGHTNESS: f32;
@group(1) @binding(3)
var<uniform> BLACKOUT_EH: u32;
@group(1) @binding(4)
var<uniform> MAX_DIST: f32;
@group(1) @binding(5)
var<uniform> DISTORTION_POWER: f32;

const MIN_DIST = 0.001;
const EPSILON_VEC: vec2<f32> = vec2<f32>(1e-3, 0.0);
const TWO_PI = 6.28318530718;
const ONE_PI = 3.14159265359;
const HALF_PI = 1.57079632679;
const MAX_ITERATIONS = 1000;

fn u32_to_bool(n: u32) -> bool {
    return n != 0u;
}

// struct Photon {
//     pos: vec3<f32>,
//     vel: vec3<f32>,
// }

// struct BlackHole {
//     pos: vec3<f32>,
//     mass: f32,
// }

//const C = 1.0;
// c is 1 for simplicity in all calculations

// fn rs(black_hole: BlackHole) -> f32 {
//     return 2.0 * G * black_hole.mass;
// }

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

    let sd_accretion_disk = sdf_accretion_disk(p, vec3<f32>(0.0), 5.0 * RS, 3.0 * RS);

    // return min(sd_sphere_bh, sd_accretion_disk);
    return sd_accretion_disk;
}

fn rd_derivative(ro: vec3<f32>, h2: f32) -> vec3<f32> {
    // return vec3<f32>(0.0);

    return DISTORTION_POWER * -1.5 * h2 * ro / pow(dot(ro, ro), 2.5);
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
        if u32_to_bool(BLACKOUT_EH) && dist_to_bh_sq < 1.0 {
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
        var delta_time = min(MAX_DELTA_TIME, dist);

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
        if distance_travelled > MAX_DIST {
            // return vec3<f32>(0.0, 1.0, 1.0);
            break;
        }        
    }
    return rd * BG_BRIGHTNESS;
}

// fn get_col(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec3<f32> {
//     let BLACK_HOLE: BlackHole = BlackHole(vec3<f32>(0.0, 0.0, 0.0), BLACK_HOLE_MASS);
    
//     var photon = Photon(ray_origin, ray_dir);
//     var distance_travelled = 0.0;
//     for (var i = 0; i < 500; i++) {

//         let dist = min(
//             get_dist(photon.pos, BLACK_HOLE),
//             sdf_sphere(photon.pos, BLACK_HOLE.pos + normalize(BLACK_HOLE.pos - ray_origin) * rs(BLACK_HOLE) * 1.5, 0.05)
//         );
//         if dist < MIN_DIST {
//             return vec3<f32>(1.0);
//         }
        
//         // can interpret dist as a time due to c = 1
//         var delta_time = min(MAX_DELTA_TIME, dist);

//         // distance between black hole and photon
//         let dist_to_bh = distance(BLACK_HOLE.pos, photon.pos);

//         // if (dist_to_bh < rs(BLACK_HOLE)) {
//         //    return vec3<f32>(0.0);
//         // }


//         // some normalised vectors relating to the photon's position in relation to the black hole
//         let photon_to_bh_norm = (BLACK_HOLE.pos - photon.pos) / dist_to_bh;
//         let photon_to_bh_perp_norm = -normalize(cross(cross(photon.vel, photon_to_bh_norm), photon_to_bh_norm));
//         // velocity if the photon were directly below the black hole
//         var converted_vel = vec2<f32>(dot(photon.vel, photon_to_bh_perp_norm), dot(photon.vel, photon_to_bh_norm));

//         // ------------------------------------------------------
//         //                T         .black hole
//         //                |
//         //            dist|         ^photon_to_bh_norm
//         //                |         |
//         //  photon_to_bh_perp_norm<-.photon
//         //                           \converted vel
//         // ------------------------------------------------------


//         // magnitude of the newtonian force with no relativistic correction
//         let newtonian_force_mag = G * BLACK_HOLE.mass / (dist_to_bh * dist_to_bh);

//         // relativistic correction
//         var theta = atan2(converted_vel.y, converted_vel.x);

//         // var delta_theta = TWO_PI * 0.001;
//         // let delta_time = max(min(delta_theta / (newtonian_force_mag * sin(theta)), dist), 0.005);
//         // delta_theta = newtonian_force_mag * delta_time * sin(theta);
//         let delta_theta = abs(delta_time * newtonian_force_mag * sin(theta)) / abs(1.0 - 2.0 * G * BLACK_HOLE.mass / dist_to_bh);

//         theta += delta_theta;
//         converted_vel = vec2<f32>(cos(theta), sin(theta));

//         photon.pos += photon.vel * delta_time * 0.5;
//         // apply the corected force
//         photon.vel = photon_to_bh_perp_norm * converted_vel.x + photon_to_bh_norm * converted_vel.y;

//         photon.pos += photon.vel * delta_time * 0.5;

//         // let photon_2D = convert_photon(photon);
//         // let force = BLACK_HOLE.pos - photon_2D.pos;
//         // let theta = -HALF_PI;
//         // let r = length(force);
//         // let fg = G * BLACK_HOLE.mass / (r * r);
//         // let deltaTheta = -fg * (DT / C) * sin()
//     }
//     return photon.vel;
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_dir = normalize(in.camera_to_vertex);
    let col = get_col(camera_pos, ray_dir);
    return vec4<f32>(col, 1.0);
}





















































const LIGHT_DIR: vec3<f32> = vec3<f32>(1.0, 1.0, -1.0);
const LIGHT_POS: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);

const DirectionalLightOption: i32 = 0;
const PointLightOption: i32 = 1;
const LIGHT_OPTION: i32 = DirectionalLightOption;

const SHADOWS_ENABLED = false;
const AMBIENT_OCCLUSION_ENABLED = false;

fn sdf_sphere(p: vec3<f32>, centre: vec3<f32>, r: f32) -> f32 {
    return length(centre - p) - r;
}

// fn sdf_cylinder(p: vec3<f32>, centre: vec3<f32>, r: f32) -> f32 {
//     let d = p - centre;
//     return length(d.xz) - r;
// }

fn sdf_torus_1(p: vec3<f32>, centre: vec3<f32>, big_r: f32, little_r: f32) -> f32 {
    let d = p - centre;
    let sd_cylinder = length(d.xz) - big_r;
    let sd_ring = sqrt(sd_cylinder * sd_cylinder + d.y * d.y);
    return sd_ring - little_r;
}

// fn sdf_weird_cylinder(p: vec3<f32>, ray_dir: vec3<f32>) -> Distance {
//     let c = vec3<f32>(0.0);
//     let r = 0.2;

//     let p_to_c = c - p;
//     let perp = cross(ray_dir, p_to_c);
//     let perp_in_plane = cross(ray_dir, perp);
//     // return perp_in_plane;
    
//     let v = normalize(perp_in_plane);
//     // return vec3<f32>(-v);
//     // let lambda = (p_to_c.x * v.y - v.x * ray_dir.y) / (ray_dir.x * v.y - v.x * p_to_c.y);
//     let lambda = dot(p_to_c, ray_dir);

//     // could minus this
//     let mu = dot(p_to_c, v);

//     // if (abs(mu) < r) {
//     //     return Distance(1.0, false);
//     // }
//     // return vec3<f32>(-mu);
    
//     // let mu = (p_to_c.x * ray_dir.y - ray_dir.x * p_to_c.y) / (v.x * ray_dir.y - ray_dir.x * p_to_c.y);
//     // return vec3<f32>(lambda);
    
//     // if lambda < 0.0 || abs(x) > 3.0 * r * RT_3_OVER_2 {
//     //     return Distance(0.0, true);
//     // }
//     // return Distance(lambda, false);
//     // return Distance(lambda + r * (1.0 / pow(cos((x * TWO_PI * RT_3_OVER_2) / (9.0 * r)), 2.0) - 1.0) - r, false);
//     let g_of_x = -r + r * (1.0 / pow(cos(mu * ONE_PI / (r * 3.0 * RT_3)), APPROACHING_ZERO) - 1.0) / (1.0 / pow(cos(ONE_PI / (2.0 * RT_3)), APPROACHING_ZERO) - 1.0);
//     if abs(mu) < r * 3.0 * RT_3_OVER_2 {
//         if g_of_x < lambda {
//             return Distance(g_of_x + lambda, false);
//             // return Distance(0.0, false);
//         }
//         // return Distance(0.5, false);
//     }
//     return Distance(0.0, true);
// }

// fn sdf_torus_2(p: vec3<f32>, centre: vec3<f32>, big_r: f32, little_r: f32) -> f32 {
//     let d = p - centre;
//     let sd_cylinder_sheet = abs(length(d.xz) - big_r);
//     let sd_sheet = abs(d.y - centre.y);
//     return max(sd_cylinder_sheet, sd_sheet);
// }

fn get_practical_distance(d: Distance) -> f32 {
    if d.is_infinte {
        return 10000.0;
    }
    return d.dist;
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

fn min_dist(d0: Distance, d1: Distance) -> Distance {
    if d0.is_infinte {
        return d1;
    }
    if d1.is_infinte {
        return d0;
    }
    if d0.dist < d1.dist {
        return d0;
    }
    return d1;
}

// fn get_normal(p: vec3<f32>, ray_dir: vec3<f32>) -> vec3<f32> {
//     let dist = get_practical_distance(get_dist_along_ray(p, ray_dir));

//     let normal = normalize(dist - vec3<f32>(
//         get_practical_distance(get_dist_along_ray(p - EPSILON_VEC.xyy, ray_dir)),
//         get_practical_distance(get_dist_along_ray(p - EPSILON_VEC.yxy, ray_dir)),
//         get_practical_distance(get_dist_along_ray(p - EPSILON_VEC.yyx, ray_dir))
//     ));
    
//     return normal;
// }

// fn get_dist_along_ray(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> Distance {
//     return min_dist(ray_march(ray_origin, ray_dir), sdf_weird_cylinder(ray_origin, ray_dir));
// }

fn ray_march(ro: vec3<f32>, rd: vec3<f32>) -> Distance {
    discard;
}

fn shadow_light_multiplier(p: vec3<f32>, normal: vec3<f32>, light_dir: vec3<f32>, light_dist_sq: Distance) -> f32 {
    if !SHADOWS_ENABLED {
        return 1.0;
    }
    let uninterrupted_light_dist = ray_march(p + 2.0 * MIN_DIST * normal, light_dir);
    if uninterrupted_light_dist.is_infinte {
        return 1.0;
    }
    let uninterrupted_light_dist_sq = Distance(
        uninterrupted_light_dist.dist *
        uninterrupted_light_dist.dist,
        false,
    );
    if !dist_less_than(uninterrupted_light_dist_sq, light_dist_sq) {
        return 1.0;
    }
    return 0.0;
}

fn get_directional_light(normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    return dot(normal, light_dir);
}

// idk what step_nbr is
// fn get_amident_occlusion(p: vec3<f32>, normal: vec3<f32>, step_dist: f32, step_nbr: f32) -> f32 {
//     if !AMBIENT_OCCLUSION_ENABLED {
//         return 1.0;
//     }
//     var _step_nbr = step_nbr;
//     var occlusion = 1.0;
//     while _step_nbr > 0.0 {
//         occlusion -= pow(_step_nbr * step_dist - get_dist(p + normal * _step_nbr * step_dist), 2.0) / _step_nbr;
//         _step_nbr -= 1.0;
//     }
//     return occlusion;
// }

fn get_light(p: vec3<f32>, normal: vec3<f32>) -> f32 {

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

    // let try_vec = vec3<f32>(-light_dir.y, light_dir.x, -light_dir.z);
    // let tangent_0 = normalize(cross(light_dir, try_vec));
    // let tangent_1 = cross(light_dir, tangent_0);

    // let a = 0.05;

    // var total_shadow_light_multiplier = 0.0;

    // let n = 10.0;
    // for (var i = 0.0; i < n + 0.5; i += 1.0) {
    //     let theta = TWO_PI * i / n;
    //     let curr_tangent = cos(theta) * tangent_0 + sin(theta) * tangent_1;
    //     let curr_wonky_light_dir = normalize(normal + curr_tangent * a);
    //     total_shadow_light_multiplier += shadow_light_multiplier(p, normal, curr_wonky_light_dir, light_dist_sq);
    // }
    // total_shadow_light_multiplier /= n;

    ////////////////////////////////////////////////////////////////////////////////////////
    
    // let tangent_2 = -0.5 * tangent_0 + RT_3_OVER_2 * tangent_1;
    // let tangent_3 = -0.5 * tangent_0 - RT_3_OVER_2 * tangent_1;

    // let normal_0 = normalize(light_dir + tangent_0 * a);
    // let normal_1 = normalize(light_dir + tangent_2 * a);
    // let normal_2 = normalize(light_dir + tangent_3 * a);

    // let total_shadow_light_multiplier = (
    //     shadow_light_multiplier(p, normal, normal_0, light_dist_sq) +
    //     shadow_light_multiplier(p, normal, normal_1, light_dist_sq) +
    //     shadow_light_multiplier(p, normal, normal_2, light_dist_sq)
    // ) / 3.0;

    let total_shadow_light_multiplier = 1.0;//shadow_light_multiplier(p, normal, light_dir, light_dist_sq);

    let ambient_occlusion_multiplier = 1.0;//pow(get_amident_occlusion(p, normal, 0.015, 20.0), 40.0);

    return ambient_occlusion_multiplier * total_shadow_light_multiplier * get_directional_light(normal, light_dir);
}

// fn ray_march(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> Distance {
//     var dist_travelled = 0.0;
//     for (var i = 0; i < MAX_STEPS; i++) {
//         let p = ray_origin + dist_travelled * ray_dir;
//         let surface_dist = get_dist(p);
//         dist_travelled += surface_dist;
//         // check for infinite distance
//         if dist_travelled > MAX_DIST {
//             break;
//         }
//         // check for hit surface
//         if surface_dist < MIN_DIST {
//             return Distance(dist_travelled, false);
//         }
//     }
//     // this represents an infinite distance
//     return Distance(0.0, true);
// }