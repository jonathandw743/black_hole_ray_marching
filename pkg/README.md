# Black Hole Ray Marching

This app uses ray marching, some maths from here and here and the Runge-Kutta method to simulate light rays
around a black hole.

I started this app using the the [Learn Wgpu tutorial](https://sotrh.github.io/learn-wgpu/). The user can control the camera and some settings.

Bloom using a Kawase dual filter is also implemented.

Using rust and WGPU (wgpu-rs), and wgsl shaders. Supports WASM.

Use `cargo run` to start.

![Black hole](images/black_hole_better_bloom.png)

## Controls

- WASD, F and Space to move.
- Q and E to change speed.
- Press number keys and arrows to change shader uniforms.
- Hold F and the number keys to change maximum framerate.

## Other

Uses some maths from:

- http://rantonels.github.io/starless/
- https://github.com/rantonels/starless/blob/master/tracer.py