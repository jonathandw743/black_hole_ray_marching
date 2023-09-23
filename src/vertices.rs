

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
