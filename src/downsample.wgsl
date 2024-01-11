#import <./screen_triangle_vertex_output.wgsl>

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSampleLevel(input_texture, texture_sampler, in.texcoord, 0.0);
    return textureStore(output_texture, pos, col);
}