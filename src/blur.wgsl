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


fn linear(t: f32) -> f32 {
    return t;
}

fn cubic(t: f32) -> f32 {
    return 3.0 * t * t - 2.0 * t * t * t;
}

fn reversecubic(t: f32) -> f32 {
    if t < 0.5 {
        return sqrt(0.5 * t);
    } else {
        return 1.0 - sqrt(0.5 - 0.5 * t);
    }
}

fn linearmix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * linear(t);
}

fn cubicmix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * cubic(t);
}

fn reversecubicmix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * reversecubic(t);
}

fn mymix(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return linearmix(a, b, t);
}

// this will always do linear
fn foo(fractpp: vec2<f32>, blur_amount: f32, floorcol: vec3<f32>, nextcolx: vec3<f32>, nextcoly: vec3<f32>, nextcol: vec3<f32>, actual_col: vec3<f32>) -> vec3<f32> {

    var fpp = fractpp;
    var fc = floorcol;
    var ncx = nextcolx;
    var ncy = nextcoly;
    var nc = nextcol;


    for (var i = 0u; i < u32(blur_amount); i++) {
        var currfpp = vec2<f32>(0.0);
        var currfc = vec3<f32>(0.0);
        var currncx = vec3<f32>(0.0);
        var currncy = vec3<f32>(0.0);
        var currnc = vec3<f32>(0.0);
        if fpp.x < 0.5 {
            if fpp.y < 0.5 {
                currfpp = (fpp - vec2<f32>(0.0, 0.0)) * 2.0;
                currfc = fc;
                currncx = mymix(fc, ncx, 0.5);
                currncy = mymix(fc, ncy, 0.5);
                currnc = mymix(mymix(fc, ncx, 0.5), mymix(ncy, nc, 0.5), 0.5);
            } else {
                currfpp = (fpp - vec2<f32>(0.0, 0.5)) * 2.0;
                currfc = mymix(fc, ncy, 0.5);
                currncx = mymix(mymix(fc, ncx, 0.5), mymix(ncy, nc, 0.5), 0.5);
                currncy = ncy;
                currnc = mymix(ncy, nc, 0.5);
            }
        } else {
            if fpp.y < 0.5 {
                currfpp = (fpp - vec2<f32>(0.5, 0.0)) * 2.0;
                currfc = mymix(fc, ncx, 0.5);
                currncx = ncx;
                currncy = mymix(mymix(fc, ncx, 0.5), mymix(ncy, nc, 0.5), 0.5);
                currnc = mymix(ncx, nc, 0.5);
            } else {
                currfpp = (fpp - vec2<f32>(0.5, 0.5)) * 2.0;
                currfc = mymix(mymix(fc, ncx, 0.5), mymix(ncy, nc, 0.5), 0.5);
                currncx = mymix(ncx, nc, 0.5);
                currncy = mymix(ncy, nc, 0.5);
                currnc = nc;
            }
        }
        fpp = currfpp;
        fc = currfc;
        ncx = currncx;
        ncy = currncy;
        nc = currnc;
    }

    return fc;

    // if blur_amount == 0.0 {
    //     return floorcol;
    // }
    // if fract_pp.x < 0.5 {
    //     if fract_pp.y < 0.5 {
    //         return foo((fract_pp - vec2<f32>(0.0, 0.0)) * 2.0, blur_amount - 1.0, floorcol, mix(floorcol, nextcolx, 0.5), mix(floorcol, nextcoly, 0.5), mix(floorcol, nextcol, 0.5));
    //     } else {
    //         return foo((fract_pp - vec2<f32>(0.0, 0.5)) * 2.0, blur_amount - 1.0, mix(floorcol, nextcoly, 0.5), mix(floorcol, nextcol, 0.5), nextcoly, mix(nextcoly, nextcol, 0.5));
    //     }
    // } else {
    //     if fract_pp.y < 0.5 {
    //         return foo((fract_pp - vec2<f32>(0.5, 0.0)) * 2.0, blur_amount - 1.0, mix(floorcol, nextcolx, 0.5), nextcolx, mix(floorcol, nextcol, 0.5), mix(nextcolx, nextcol, 0.5));
    //     } else {
    //         return foo((fract_pp - vec2<f32>(0.5, 0.5)) * 2.0, blur_amount - 1.0, mix(floorcol, nextcol, 0.5), mix(nextcolx, nextcol, 0.5), mix(nextcoly, nextcol, 0.5), nextcol);
    //     }
    // }
}

fn map_col_component(x: f32) -> f32 {
    return 1.0 / (1.0 - x) - 1.0;
}

fn map_col(col: vec3<f32>) -> vec3<f32> {
    var r = col.x;
    if (r < 0.5) {
        r = 0.0;
    }
    var g = col.y;
    if (g < 0.5) {
        g = 0.0;
    }
    var b = col.z;
    if (b < 0.5) {
        b = 0.0;
    }
    return vec3<f32>(map_col_component(r), map_col_component(g), map_col_component(b));
}

const PI: f32 = 3.14159265359;
const ANGLE_INC: f32 = 0.5;
const BLUR_SIZE: f32 = 16.0;
const BLUR_INC: f32 = 0.2;
const POWER: f32 = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let fres = vec2<f32>(resolution);
    let pixel_pos = in.uv * fres;
    //abc
    //def
    //ghi

    // let a_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, 1.0);
    // let b_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, 1.0);
    // let c_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, 1.0);
    // let d_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, 0.0);
    // let e_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, 0.0);
    // let f_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, 0.0);
    // let g_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, -1.0);
    // let h_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, -1.0);
    // let i_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, -1.0);

    // let a = textureLoad(t_diffuse, vec2<u32>(a_pos), 0).xyz;
    // let b = textureLoad(t_diffuse, vec2<u32>(b_pos), 0).xyz;
    // let c = textureLoad(t_diffuse, vec2<u32>(c_pos), 0).xyz;
    // let d = textureLoad(t_diffuse, vec2<u32>(d_pos), 0).xyz;
    // let e = textureLoad(t_diffuse, vec2<u32>(e_pos), 0).xyz;
    // let f = textureLoad(t_diffuse, vec2<u32>(f_pos), 0).xyz;
    // let g = textureLoad(t_diffuse, vec2<u32>(g_pos), 0).xyz;
    // let h = textureLoad(t_diffuse, vec2<u32>(h_pos), 0).xyz;
    // let i = textureLoad(t_diffuse, vec2<u32>(i_pos), 0).xyz;

    // let col = (a + 2.0 * b + c + 2.0 * d + 4.0 * e + 2.0 * f + g + 2.0 * h + i) / 16.0;

    var blurcol = vec3<f32>(0.0);
    var d = 0.0;
    for (var a = -PI; a < PI; a += ANGLE_INC) {
        for (var r = 0.0; r < 1.0; r += BLUR_INC) {
            let relpos = vec2<f32>(r * BLUR_SIZE * cos(a), r * BLUR_SIZE * sin(a));
            let pos = pixel_pos + relpos;
            // if pos.x < 0.0 || pos.y < 0.0 || pos.x >= fres.x || pos.y >= fres.y {
            //     continue;
            // }
            // let s = (x * x + y * y) / (BLUR_AMOUNT * BLUR_AMOUNT);
            // if s > 1.0 {
            //     continue;
            // }
            // let strength = s * s - 2.0 * s + 1.0;
            // let strength = 2.0 * r * r * r - 3.0 * r * r;
            let currcol = map_col(textureLoad(t_diffuse, vec2<u32>(pos), 0).xyz);

            blurcol += currcol;
            d += 1.0;
        }
    }
    blurcol /= d;

    let col = textureLoad(t_diffuse, vec2<u32>(pixel_pos), 0).xyz + POWER * blurcol;

    // let fractpp = fract(pixel_pos);
    // let floorpp = vec2<u32>(floor(pixel_pos) * blur_factor);
    // let nextpp = floorpp + vec2<u32>(vec2<f32>(blur_factor, blur_factor));

    // let floorcol = textureLoad(t_diffuse, floorpp, 0).xyz;
    // let nextcolx = textureLoad(t_diffuse, vec2<u32>(nextpp.x, floorpp.y), 0).xyz;
    // let nextcoly = textureLoad(t_diffuse, vec2<u32>(floorpp.x, nextpp.y), 0).xyz;
    // let nextcol = textureLoad(t_diffuse, nextpp, 0).xyz;
    // let colx1 = mymix(floorcol, nextcolx, fractpp.x);
    // let colx2 = mymix(nextcoly, nextcol, fractpp.x);
    // let col = mymix(colx1, colx2, fractpp.y);
    // let col = foo(fractpp, BLUR_AMOUNT, floorcol, nextcolx, nextcoly, nextcol, textureLoad(t_diffuse, vec2<u32>(in.uv * vec2<f32>(resolution)), 0).xyz);
    return vec4<f32>(col, 1.0);
}