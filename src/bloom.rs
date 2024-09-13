use crate::{blur::Blur, copy::Copy, remix::Remix};

pub struct Bloom {
    pub blurs: Vec<Blur>,
    pub copies: Vec<Copy>,
    pub remixes: Vec<Remix>,
    pub final_remix: Remix,
    levels: usize,
}

impl Bloom {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, levels: usize) -> Self {
        let mut blurs = Vec::new();
        let mut copies = Vec::new();
        let mut remixes = Vec::new();
        for level in 1..=levels {
            blurs.push(Blur::new(device, config, level));
            copies.push(Copy::new(device, config));
            remixes.push(Remix::new(device, config));
        }
        let final_remix = Remix::new(device, config);
        Self {
            blurs,
            copies,
            remixes,
            final_remix,
            levels,
        }
    }

    pub fn full_image_input_texture_view(&self) -> &wgpu::TextureView {
        &self.final_remix.input_texture_0_view()
    }

    pub fn blackout_input_texture_view(&self) -> &wgpu::TextureView {
        &self.copies[0].input_texture_view()
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) {
        for blur in &mut self.blurs {
            blur.resize(device, config, queue);
        }
        for remix in &mut self.remixes {
            remix.resize(device, config);
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: Option<&wgpu::TextureView>,
    ) {
        for level in 0..self.levels - 1 {
            self.copies[0].render(encoder, Some(self.blurs[0].input_texture_view()));
            self.copies[0].render(encoder, Some(self.remixes[0].input_texture_0_view()));
            self.blurs[0].render(encoder, Some(self.remixes[0].input_texture_1_view()));
            self.remixes[0].render(encoder, Some(self.copies[level + 1].input_texture_view()));
        }

        self.copies[self.levels - 1].render(encoder, Some(self.blurs[self.levels - 1].input_texture_view()));
        self.copies[self.levels - 1].render(encoder, Some(self.remixes[self.levels - 1].input_texture_0_view()));
        self.blurs[self.levels - 1].render(encoder, Some(self.remixes[self.levels - 1].input_texture_1_view()));
        self.remixes[self.levels - 1].render(encoder, Some(self.final_remix.input_texture_1_view()));

        self.final_remix.render(encoder, output_view);
    }
}
