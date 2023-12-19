use bytemuck::Contiguous;

use crate::vertex::{Vertex, PostProcessingVertex};

const BOX_SIZE: f32 = 25.0;

pub const VERTICES: &[Vertex] = &[
    // back face
    Vertex {
        position: [-BOX_SIZE, BOX_SIZE, BOX_SIZE],
    },
    Vertex {
        position: [BOX_SIZE, BOX_SIZE, BOX_SIZE],
    },
    Vertex {
        position: [-BOX_SIZE, -BOX_SIZE, BOX_SIZE],
    },
    Vertex {
        position: [BOX_SIZE, -BOX_SIZE, BOX_SIZE],
    },
    // front face
    Vertex {
        position: [-BOX_SIZE, BOX_SIZE, -BOX_SIZE],
    },
    Vertex {
        position: [BOX_SIZE, BOX_SIZE, -BOX_SIZE],
    },
    Vertex {
        position: [-BOX_SIZE, -BOX_SIZE, -BOX_SIZE],
    },
    Vertex {
        position: [BOX_SIZE, -BOX_SIZE, -BOX_SIZE],
    },
];

const POSTPROCESSING_BOX_SIZE: f32 = 1.0;
const MIN_UV: f32 = 0.0;
const MAX_UV: f32 = 1.0;
pub const POSTPROCESSING_VERTICES: &[PostProcessingVertex] = &[
    PostProcessingVertex {
        position: [-POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MIN_UV, MIN_UV]
    },
    PostProcessingVertex {
        position: [-POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MIN_UV, MAX_UV],
    },
    PostProcessingVertex {
        position: [POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MAX_UV, MIN_UV],
    },
    PostProcessingVertex {
        position: [POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MAX_UV, MIN_UV],
    },
    PostProcessingVertex {
        position: [-POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MIN_UV, MAX_UV],
    },
    PostProcessingVertex {
        position: [POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE],
        pixel_uv: [MAX_UV, MAX_UV],
    },
];
