struct VertexOutput {
    // Mark output position as invariant so it's safe to use it with depth test Equal.
    // Without @invariant, different usages in different render pipelines might optimize differently,
    // causing slightly different results.
    @invariant @builtin(position)
    position: vec4f,
    @location(0)
    texcoord: vec2f,
};

@group(0) @binding(0)
var prev_texture: texture_2d<f32>;
@group(0) @binding(1)
var prev_texture_sampler: sampler;
@group(0) @binding(2)
var original_blackout_texture: texture_2d<f32>;
@group(0) @binding(3)
var original_blackout_texture_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let col_from_prev_texture = textureSampleLevel(prev_texture, prev_texture_sampler, in.texcoord, 0.0);
    let col_from_prev_texture = textureBicubic(prev_texture, prev_texture_sampler, in.texcoord);
    // let col_from_original_blackout_texture = textureSampleLevel(original_blackout_texture, original_blackout_texture_sampler, in.texcoord, 0.0);
    // let col = mix(col_from_prev_texture, col_from_original_blackout_texture, 0.05);
    let col = col_from_prev_texture;// + col_from_original_blackout_texture;
    return col;
}

// from http://www.java-gaming.org/index.php?topic=35123.0
fn cubic2(v: f32) -> vec4<f32> {
    let n = vec4<f32>(1.0, 2.0, 3.0, 4.0) - v;
    let s = n * n * n;
    let x = s.x;
    let y = s.y - 4.0 * s.x;
    let z = s.z - 4.0 * s.y + 6.0 * s.x;
    let w = 6.0 - x - y - z;
    return vec4<f32>(x, y, z, w) * (1.0/6.0);
}

fn textureBicubic(my_texture: texture_2d<f32>, my_sampler: sampler, texCoords: vec2<f32>) -> vec4<f32> {

   let texSize = vec2<f32>(textureDimensions(my_texture, 0));
   let invTexSize = 1.0 / texSize;
   
   var my_texCoords = texCoords * texSize - 0.5;

   
    let fxy = fract(my_texCoords);
    my_texCoords -= fxy;

    let xcubic = cubic2(fxy.x);
    let ycubic = cubic2(fxy.y);

    let c = my_texCoords.xxyy + vec2<f32>(-0.5, 1.5).xyxy;
    
    let s = vec4<f32>(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
    var offset = c + vec4<f32>(xcubic.yw, ycubic.yw) / s;
    
    offset *= invTexSize.xxyy;
    
    let sample0 = textureSampleLevel(my_texture, my_sampler, offset.xz, 0.0);
    let sample1 = textureSampleLevel(my_texture, my_sampler, offset.yz, 0.0);
    let sample2 = textureSampleLevel(my_texture, my_sampler, offset.xw, 0.0);
    let sample3 = textureSampleLevel(my_texture, my_sampler, offset.yw, 0.0);

    let sx = s.x / (s.x + s.y);
    let sy = s.z / (s.z + s.w);

    return mix(
       mix(sample3, sample2, sx), mix(sample1, sample0, sx)
    , sy);
}
