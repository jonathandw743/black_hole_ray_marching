// use crate::{kawase_downsampling::KawaseDownsampling, kawase_mixing_upsampling::KawaseMixingUpsampling};

use crate::{kawase_downsampling::KawaseDownsampling, kawase_upsampling::KawaseUpsampling};

pub struct Blur {
    pub downsampling: KawaseDownsampling,
    pub upsampling: KawaseUpsampling,
    levels: usize,
}

impl Blur {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, levels: usize) -> Self {
        let downsampling = KawaseDownsampling::new(device, config, levels);
        let upsampling = KawaseUpsampling::new(device, config, levels);

        Self {
            downsampling,
            upsampling,
            levels,
        }
    }

    pub fn input_texture_view(&self) -> &wgpu::TextureView {
        &self.downsampling.input_texture_view()
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) {
        self.downsampling.resize(device, config, queue);
        self.upsampling.resize(device, config, queue);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: Option<&wgpu::TextureView>,
    ) {
        self.downsampling
            .render(encoder, Some(self.upsampling.input_texture_view()));
        self.upsampling.render(encoder, output_view);
    }
}
