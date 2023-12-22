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

const BLUR_AMOUNT: f32 = 1.0;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_pos = in.uv * vec2<f32>(resolution);
    //abc
    //def
    //ghi

    let a_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, 1.0);
    let b_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, 1.0);
    let c_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, 1.0);
    let d_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, 0.0);
    let e_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, 0.0);
    let f_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, 0.0);
    let g_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(-1.0, -1.0);
    let h_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(0.0, -1.0);
    let i_pos = pixel_pos + BLUR_AMOUNT * vec2<f32>(1.0, -1.0);

    let a = textureLoad(t_diffuse, vec2<u32>(a_pos), 0).xyz;
    let b = textureLoad(t_diffuse, vec2<u32>(b_pos), 0).xyz;
    let c = textureLoad(t_diffuse, vec2<u32>(c_pos), 0).xyz;
    let d = textureLoad(t_diffuse, vec2<u32>(d_pos), 0).xyz;
    let e = textureLoad(t_diffuse, vec2<u32>(e_pos), 0).xyz;
    let f = textureLoad(t_diffuse, vec2<u32>(f_pos), 0).xyz;
    let g = textureLoad(t_diffuse, vec2<u32>(g_pos), 0).xyz;
    let h = textureLoad(t_diffuse, vec2<u32>(h_pos), 0).xyz;
    let i = textureLoad(t_diffuse, vec2<u32>(i_pos), 0).xyz;

    let col = (a + 2.0 * b + c + 2.0 * d + 4.0 * e + 2.0 * f + g + 2.0 * h + i) / 16.0;

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