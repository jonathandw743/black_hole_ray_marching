
// #[allow(dead_code)]
// #[allow(unused)]

use black_hole_ray_marching::run;




fn main() {
    // println!("{}", include_str!("../shaders/raymarching.wgsl"));
    pollster::block_on(run());
    // println!("{}", env!("CARGO_MANIFEST_DIR"));
    // run();

}
