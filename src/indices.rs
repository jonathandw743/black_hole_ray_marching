#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    //outside
    // // back face
    // 0,1,2, 1,3,2,
    // // front face
    // 4,6,5, 5,6,7,
    // // left face
    // 0,2,4, 4,2,6,
    // // right face
    // 5,7,3, 5,3,1,
    // // top face
    // 1,0,4, 1,4,5,
    // // bottom face
    // 7,6,3, 6,2,3,

    //inside
    // back face
    2,1,0, 2,3,1,
    // front face
    5,6,4, 7,6,5,
    // left face
    4,2,0, 6,2,4,
    // right face
    3,7,5, 1,3,5,
    // top face
    4,0,1, 5,4,1,
    // bottom face
    3,6,7, 3,2,6,
];
