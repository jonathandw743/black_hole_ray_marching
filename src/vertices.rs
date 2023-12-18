use crate::vertex::Vertex;

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

const POSTPROCESSING_BOX_SIZE: f32 = 0.4;
pub const POSTPROCESSING_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE, 0.0],
    },
    Vertex {
        position: [-POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE, 0.0],
    },
    Vertex {
        position: [POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE, 0.0],
    },
    Vertex {
        position: [POSTPROCESSING_BOX_SIZE, POSTPROCESSING_BOX_SIZE, 0.0],
    },
    Vertex {
        position: [-POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE, 0.0],
    },
    Vertex {
        position: [POSTPROCESSING_BOX_SIZE, -POSTPROCESSING_BOX_SIZE, 0.0],
    },
];
