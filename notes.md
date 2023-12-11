there is discrepancy between the speed of the desktop version and the wasm version - check the deltatime time control, framerates etc
```rs
struct OtherUniform<T>
where
    T: ShaderType,
{
    label: String,
    shader_stage: ShaderStages,
    value: T,
}

trait OtherUniformTrait {
    fn get_label(&self) -> &String;
    fn get_shader_stage(&self) -> ShaderStages;
}

struct OtherUniforms<const N: usize> {
    positive_modifier_key_code: VirtualKeyCode,
    negative_modifier_key_code: VirtualKeyCode,
    other_uniforms: [Box<dyn OtherUniformTrait>; N],
}

impl<const N: usize> OtherUniforms<N> {
    pub fn new(
        positive_modifier_key_code: VirtualKeyCode,
        negative_modifier_key_code: VirtualKeyCode,
        other_uniforms: [Box<dyn OtherUniformTrait>; N],
    ) -> Self {
        Self {}
    }
}
```