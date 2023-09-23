
// #[allow(dead_code)]
// #[allow(unused)]

use bh::run;



fn main() {
    // println!("{}", include_str!("../shaders/raymarching.wgsl"));
    pollster::block_on(run());
    // println!("{}", env!("CARGO_MANIFEST_DIR"));
    // run();
}
