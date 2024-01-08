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
var s_diffuse: sampler;
@group(0) @binding(2)
var<uniform> resolution: vec2<u32>;
@group(0) @binding(3)
var<uniform> blur_size: f32;

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

fn weirdcubicfunction(v: f32) -> vec4<f32> {
    let n = vec4<f32>(1.0, 2.0, 3.0, 4.0) - v;
    let s = n * n * n;
    let x = s.x;
    let y = s.y - 4.0 * s.x;
    let z = s.z - 4.0 * s.y + 6.0 * s.x;
    let w = 6.0 - x - y - z;
    return vec4<f32>(x, y, z, w) * (1.0/6.0);
}

fn my_textureSample(t_diffuse: texture_2d<f32>, p: vec2<f32>, blur_size: f32) -> vec3<f32> {
    // the position of the pixel in the downsample
    // this has a fractional component
    let pp = p / blur_size; // might have to - 0.5
    // where the pixel is inside the downsample's pixel
    let fract_pp = fract(pp);
    // this does not have a fractional component so identifies a pixel in the downsample
    let floor_pp = vec2<i32>(floor(pp) * blur_size);
    
    var s0 = map_col_one_to_infinity(textureLoad(t_diffuse, vec2<i32>(floor_pp), 0).xyz);
    if dot(s0, s0) < 1.0 {
        s0 = vec3<f32>(0.0);
    }
    var s1 = map_col_one_to_infinity(textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(blur_size, 0.0)), 0).xyz);
    if dot(s1, s1) < 1.0 {
        s1 = vec3<f32>(0.0);
    }
    var s2 = map_col_one_to_infinity(textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(0.0, blur_size)), 0).xyz);
    if dot(s2, s2) < 1.0 {
        s2 = vec3<f32>(0.0);
    }
    var s3 = map_col_one_to_infinity(textureLoad(t_diffuse, vec2<i32>(floor_pp) + vec2<i32>(vec2<f32>(blur_size, blur_size)), 0).xyz);
    if dot(s3, s3) < 1.0 {
        s3 = vec3<f32>(0.0);
    }

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
const BLUR_INC: f32 = 0.2;
const POWER: f32 = 0.1;

fn map_col_component_one_to_infinity(component: f32) -> f32 {
    return 1.0 / (1.0 - component) - 1.0;
}

fn map_col_one_to_infinity(col: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(map_col_component_one_to_infinity(col.x), map_col_component_one_to_infinity(col.y), map_col_component_one_to_infinity(col.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let fres = vec2<f32>(resolution);
    let pixel_pos = in.uv * fres;

    let actual_col = map_col_one_to_infinity(textureLoad(t_diffuse, vec2<i32>(pixel_pos), 0).xyz);
    let blurred_col = my_textureBicubic(t_diffuse, pixel_pos, blur_size);

    let col = actual_col + blurred_col * 0.5;
    
    return vec4<f32>(col, 1.0);
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