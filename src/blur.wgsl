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
    return cubicmix(a, b, t);
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

// from http://www.java-gaming.org/index.php?topic=35123.0
// fn cubic2(v: f32) -> vec4<f32> {
//     let n = vec4<f32>(1.0, 2.0, 3.0, 4.0) - v;
//     let s = n * n * n;
//     let x = s.x;
//     let y = s.y - 4.0 * s.x;
//     let z = s.z - 4.0 * s.y + 6.0 * s.x;
//     let w = 6.0 - x - y - z;
//     return vec4<f32>(x, y, z, w) * (1.0/6.0);
// }

// fn textureBicubic(my_sampler: texture_2d<f32>, texCoords: vec2<f32>) -> vec4<f32> {

//    let texSize = textureSize(my_sampler, 0);
//    let invTexSize = 1.0 / texSize;
   
//    var my_texCoords = texCoords * texSize - 0.5;

   
//     let fxy = fract(my_texCoords);
//     my_texCoords -= fxy;

//     let xcubic = cubic2(fxy.x);
//     let ycubic = cubic2(fxy.y);

//     let c = my_texCoords.xxyy + vec2<f32>(-0.5, 1.5).xyxy;
    
//     let s = vec4<f32>(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
//     var offset = c + vec4<f32>(xcubic.yw, ycubic.yw) / s;
    
//     offset *= invTexSize.xxyy;
    
//     let sample0 = textureLoad(my_sampler, offset.xz, 0);
//     let sample1 = textureLoad(my_sampler, offset.yz, 0);
//     let sample2 = textureLoad(my_sampler, offset.xw, 0);
//     let sample3 = textureLoad(my_sampler, offset.yw, 0);

//     let sx = s.x / (s.x + s.y);
//     let sy = s.z / (s.z + s.w);

//     return mix(
//        mix(sample3, sample2, sx), mix(sample1, sample0, sx)
//     , sy);
// }

fn cubic2(heights: vec4<f32>, x: f32) -> f32 {
    // let A = mat3x3<f32> (
    //     1, 1, 1,
    //     8, 4, 2,
    //     27, 9, 3,
    // );

    let A_inv = mat3x3<f32>(
        3.0, -3.0, 1.0,
        -15.0, 12.0, -3.0,
        18.0, -9.0, 2.0
    ) / 6.0;

    let d = heights.x;

    let rel_heights = (heights - d).yzw;

    let unknowns = vec3<f32>(dot(A_inv[0], rel_heights), dot(A_inv[1], rel_heights), dot(A_inv[2], rel_heights));

    return unknowns.x * x * x * x + unknowns.y * x * x + unknowns.z * x + d;
}

fn cubic2_2d(heights: mat4x4<f32>, x: vec2<f32>) -> f32 {
    let h0 = cubic2(heights[0], x.x);
    let h1 = cubic2(heights[1], x.x);
    let h2 = cubic2(heights[2], x.x);
    let h3 = cubic2(heights[3], x.x);

    return cubic2(vec4<f32>(h0, h1, h2, h3), x.y);

}

fn weirdcubicfunction(v: f32) -> vec4<f32> {
    let n = vec4<f32>(1.0, 2.0, 3.0, 4.0) - v;
    let s = n * n * n;
    let x = s.x;
    let y = s.y - 4.0 * s.x;
    let z = s.z - 4.0 * s.y + 6.0 * s.x;
    let w = 6.0 - x - y - z;
    return vec4<f32>(x, y, z, w) * (1.0/6.0);
}

fn textureBicubic(s_diffuse: sampler, t_diffuse: texture_2d<f32>, texCoords: vec2<f32>) -> vec4<f32> {
    // get the size of the t_diffuse
    // let texSize = t_diffuse.size;
    let texSize = vec2<f32>(100.0, 100.0);
    // and its inverse
    let invTexSize = 1.0 / texSize;

    // get the pixel pos corresponding to the input texCoords
    let pixel_pos = texCoords * texSize - 0.5;

    // where the sample is in the square (like fract pp)
    let fract_pp = fract(pixel_pos);
    // the pixel pos to use in sampling (like floor pp)
    let floor_pp = floor(pixel_pos);

    // ok then, idk
    // maybe like getting t from a linear t with some interpolation function?
    let xcubic = weirdcubicfunction(fract_pp.x);
    let ycubic = weirdcubicfunction(fract_pp.y);

    // idk
    // the centre coordinates of each pixel to sample of each 
    let c = vec4<f32>(floor_pp.x - 0.5, floor_pp.x + 1.5, floor_pp.y - 0.5, floor_pp.y + 1.5);

    // idk
    let s_part_1 = xcubic.xz + xcubic.yw;
    let s_part_2 = ycubic.xz + ycubic.yw;
    let s = vec4<f32>(s_part_1.x, s_part_1.y, s_part_2.x, s_part_2.y);
    
    // idk
    var offset = c + vec4<f32>(xcubic.yw, ycubic.yw) / s;

    // mapping this offset back to 0 to 1 uv space for the t_diffuseSample fucntion
    offset *= invTexSize.xxyy;

    // the actual samples (only 4, that's good compared to the 16 for 'real' bicubic interpolation)
    let sample0 = textureSample(t_diffuse, s_diffuse, offset.xz);
    let sample1 = textureSample(t_diffuse, s_diffuse, offset.yz);
    let sample2 = textureSample(t_diffuse, s_diffuse, offset.xw);
    let sample3 = textureSample(t_diffuse, s_diffuse, offset.yw);

    // idk
    let sx = s.x / (s.x + s.y);
    let sy = s.z / (s.z + s.w);

    // linear mixing between some places
    // this is a 2d mix accross a 2d space
    return mix(
        mix(sample3, sample2, sx), mix(sample1, sample0, sx)
, sy);
}

fn my_textureSample(t_diffuse: texture_2d<f32>, p: vec2<f32>, blur_size: f32) -> vec3<f32> {
    // the position of the pixel in the downsample
    // this has a fractional component
    let pp = p / blur_size; // might have to - 0.5
    // where the pixel is inside the downsample's pixel
    let fract_pp = fract(pp);
    // this does not have a fractional component so identifies a pixel in the downsample
    let floor_pp = vec2<i32>(floor(pp) * blur_size);
    
    let s0 = textureLoad(t_diffuse, vec2<i32>(floor_pp), 0).xyz;
    let s1 = textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(blur_size, 0.0)), 0).xyz;
    let s2 = textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(0.0, blur_size)), 0).xyz;
    let s3 = textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(blur_size, blur_size)), 0).xyz;

    return mix(
        mix(s0, s1, fract_pp.x), mix(s2, s3, fract_pp.x), fract_pp.y
    );
}

// 'the downsample' is texture if it were made smaller by a factor of blur_size
// texture is the texture to sample
// pixel_pos is the position of the current pixel
// (0, 0) to texture size in pixels
// blur_size is how many times smaller the downsample is
fn my_textureBicubic(t_diffuse: texture_2d<f32>, pixel_pos: vec2<f32>, blur_size: f32) -> vec3<f32> {
    // the position of the pixel in the downsample
    // this has a fractional component
    let pp = pixel_pos / blur_size;
    // where the pixel is inside the downsample's pixel
    let fract_pp = fract(pp);
    // this does not have a fractional component so identifies a pixel in the downsample
    let floor_pp = floor(pp);
    
    let x_cubic = weirdcubicfunction(fract_pp.x);
    let y_cubic = weirdcubicfunction(fract_pp.y);

    let c = vec4<f32>(floor_pp.x - 1.0, floor_pp.x + 1.0, floor_pp.y - 1.0, floor_pp.y + 1.0);

    let s = vec4<f32>(x_cubic.xz + x_cubic.yw, y_cubic.xz + y_cubic.yw);

    let offset = blur_size * (c + vec4<f32>(x_cubic.yw, y_cubic.yw) / s);

    let sample_0 = my_textureSample(t_diffuse, offset.xz, blur_size);
    let sample_1 = my_textureSample(t_diffuse, offset.yz, blur_size);
    let sample_2 = my_textureSample(t_diffuse, offset.xw, blur_size);
    let sample_3 = my_textureSample(t_diffuse, offset.yw, blur_size);

    let sx = s.x / (s.x + s.y);
    let sy = s.z / (s.z + s.w);

    return mix(
        mix(sample_3, sample_2, sx), mix(sample_1, sample_0, sx),
        sy
    );
}

const PI: f32 = 3.14159265359;
const ANGLE_INC: f32 = 0.5;
const BLUR_SIZE: f32 = 64.0;
const BLUR_INC: f32 = 0.2;
const POWER: f32 = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let maxpp = resolution - vec2<u32>(1u);
    let fres = vec2<f32>(resolution);
    let pixel_pos = in.uv * fres;// / BLUR_SIZE;

    let col = my_textureBicubic(t_diffuse, pixel_pos, BLUR_SIZE);
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

    // var blurcol = vec3<f32>(0.0);
    // var d = 0.0;
    // for (var a = -PI; a < PI; a += ANGLE_INC) {
    //     for (var r = 0.0; r < 1.0; r += BLUR_INC) {
    //         let relpos = vec2<f32>(r * BLUR_SIZE * cos(a), r * BLUR_SIZE * sin(a));
    //         let pos = pixel_pos + relpos;
    //         // if pos.x < 0.0 || pos.y < 0.0 || pos.x >= fres.x || pos.y >= fres.y {
    //         //     continue;
    //         // }
    //         // let s = (x * x + y * y) / (BLUR_AMOUNT * BLUR_AMOUNT);
    //         // if s > 1.0 {
    //         //     continue;
    //         // }
    //         // let strength = s * s - 2.0 * s + 1.0;
    //         // let strength = 2.0 * r * r * r - 3.0 * r * r;
    //         let currcol = map_col(textureLoad(t_diffuse, vec2<u32>(pos), 0).xyz);

    //         blurcol += currcol;
    //         d += 1.0;
    //     }
    // }
    // blurcol /= d;

    // let col = textureLoad(t_diffuse, vec2<u32>(pixel_pos), 0).xyz + POWER * blurcol;

    // let fractpp = fract(pixel_pos);

    // let floorpp = clamp(vec2<i32>(floor(pixel_pos) * BLUR_SIZE), vec2<i32>(0), vec2<i32>(maxpp));
    // let ceilpp = clamp(floorpp + vec2<i32>(vec2<f32>(BLUR_SIZE, BLUR_SIZE)), vec2<i32>(0), vec2<i32>(maxpp));

    // let ppminus1 = clamp(floorpp - vec2<i32>(vec2<f32>(BLUR_SIZE, BLUR_SIZE)), vec2<i32>(0), vec2<i32>(maxpp));
    // let ppplus1 = clamp(ceilpp + vec2<i32>(vec2<f32>(BLUR_SIZE, BLUR_SIZE)), vec2<i32>(0), vec2<i32>(maxpp));

    // let a0 = textureLoad(t_diffuse, vec2<i32>(ppminus1.x, ppminus1.y), 0).xyz;
    // let b0 = textureLoad(t_diffuse, vec2<i32>(floorpp.x, ppminus1.y), 0).xyz;
    // let c0 = textureLoad(t_diffuse, vec2<i32>(ceilpp.x, ppminus1.y), 0).xyz;
    // let d0 = textureLoad(t_diffuse, vec2<i32>(ppplus1.x, ppminus1.y), 0).xyz;

    // let a1 = textureLoad(t_diffuse, vec2<i32>(ppminus1.x, floorpp.y), 0).xyz;
    // let b1 = textureLoad(t_diffuse, vec2<i32>(floorpp.x, floorpp.y), 0).xyz;
    // let c1 = textureLoad(t_diffuse, vec2<i32>(ceilpp.x, floorpp.y), 0).xyz;
    // let d1 = textureLoad(t_diffuse, vec2<i32>(ppplus1.x, floorpp.y), 0).xyz;

    // let a2 = textureLoad(t_diffuse, vec2<i32>(ppminus1.x, ceilpp.y), 0).xyz;
    // let b2 = textureLoad(t_diffuse, vec2<i32>(floorpp.x, ceilpp.y), 0).xyz;
    // let c2 = textureLoad(t_diffuse, vec2<i32>(ceilpp.x, ceilpp.y), 0).xyz;
    // let d2 = textureLoad(t_diffuse, vec2<i32>(ppplus1.x, ceilpp.y), 0).xyz;

    // let a3 = textureLoad(t_diffuse, vec2<i32>(ppminus1.x, ppplus1.y), 0).xyz;
    // let b3 = textureLoad(t_diffuse, vec2<i32>(floorpp.x, ppplus1.y), 0).xyz;
    // let c3 = textureLoad(t_diffuse, vec2<i32>(ceilpp.x, ppplus1.y), 0).xyz;
    // let d3 = textureLoad(t_diffuse, vec2<i32>(ppplus1.x, ppplus1.y), 0).xyz;

    // let x_input_for_cubic = fractpp + vec2<f32>(1.0);
    // let col = vec3<f32>(cubic2_2d(mat4x4<f32>(
    //     a0.x, b0.x, c0.x, d0.x,
    //     a1.x, b1.x, c1.x, d1.x,
    //     a2.x, b2.x, c2.x, d2.x,
    //     a3.x, b3.x, c3.x, d3.x,
    // ), x_input_for_cubic),
    // cubic2_2d(mat4x4<f32>(
    //     a0.y, b0.y, c0.y, d0.y,
    //     a1.y, b1.y, c1.y, d1.y,
    //     a2.y, b2.y, c2.y, d2.y,
    //     a3.y, b3.y, c3.y, d3.y,
    // ), x_input_for_cubic),
    // cubic2_2d(mat4x4<f32>(
    //     a0.z, b0.z, c0.z, d0.z,
    //     a1.z, b1.z, c1.z, d1.z,
    //     a2.z, b2.z, c2.z, d2.z,
    //     a3.z, b3.z, c3.z, d3.z,
    // ), x_input_for_cubic));

    // let asd = 1.0;
    // col = vec3<f32>(cubic2(vec4<f32>(
    //     a0.x, b0.x, c0.x, d0.x,
    // ), fractpp.x + asd),
    // cubic2(vec4<f32>(
    //     a0.y, b0.y, c0.y, d0.y,
    // ), fractpp.x + asd),
    // cubic2(vec4<f32>(
    //     a0.z, b0.z, c0.z, d0.z,
    // ), fractpp.x + asd));

    // col = vec3<f32>(cubic2(vec4<f32>(0.0, 1.0, 2.0, 3.0), 1.5) - 13.0);
    // let floorcol = textureLoad(t_diffuse, floorpp, 0).xyz;
    // let nextcolx = textureLoad(t_diffuse, vec2<u32>(nextpp.x, floorpp.y), 0).xyz;
    // let nextcoly = textureLoad(t_diffuse, vec2<u32>(floorpp.x, nextpp.y), 0).xyz;
    // let nextcol = textureLoad(t_diffuse, nextpp, 0).xyz;
    // let colx1 = mymix(floorcol, nextcolx, fractpp.x);
    // let colx2 = mymix(nextcoly, nextcol, fractpp.x);
    // let col = mymix(colx1, colx2, fractpp.y);

//     let texSize = textureSize(my_sampler, 0);
//    let invTexSize = 1.0 / texSize;
   
//    var my_texCoords = texCoords * texSize - 0.5;

   
//     let fxy = fract(my_texCoords);
//     my_texCoords -= fxy;

//     let xcubic = cubic2(fxy.x);
//     let ycubic = cubic2(fxy.y);

//     let c = my_texCoords.xxyy + vec2<f32>(-0.5, 1.5).xyxy;
    
//     let s = vec4<f32>(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
//     var offset = c + vec4<f32>(xcubic.yw, ycubic.yw) / s;
    
//     offset *= invTexSize.xxyy;
    
//     let sample0 = textureLoad(my_sampler, offset.xz, 0);
//     let sample1 = textureLoad(my_sampler, offset.yz, 0);
//     let sample2 = textureLoad(my_sampler, offset.xw, 0);
//     let sample3 = textureLoad(my_sampler, offset.yw, 0);

//     let sx = s.x / (s.x + s.y);
//     let sy = s.z / (s.z + s.w);

//     return mix(
//        mix(sample3, sample2, sx), mix(sample1, sample0, sx)
//     , sy);

    // let col = foo(fractpp, BLUR_AMOUNT, floorcol, nextcolx, nextcoly, nextcol, textureLoad(t_diffuse, vec2<u32>(in.uv * vec2<f32>(resolution)), 0).xyz);
    return vec4<f32>(col, 1.0);
}