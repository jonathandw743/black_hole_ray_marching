use black_hole_ray_marching::run;

fn main() {
    pollster::block_on(run());

    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
