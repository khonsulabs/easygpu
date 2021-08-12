use figures::SizedRect;

use crate::{
    binding::Bind, buffers::DepthBuffer, canvas::Canvas, color::Bgra8, device::Device,
    renderer::RenderTarget, texture::Texture, transform::ScreenSpace,
};
/// Off-screen framebuffer. Can be used as a render target in render passes.
#[derive(Debug)]
pub struct Framebuffer {
    pub texture: Texture,
    pub depth: DepthBuffer,
}

impl Framebuffer {
    /// Size in pixels of the framebuffer.
    pub fn size(&self) -> usize {
        self.texture.size.cast::<usize>().area().get()
    }

    /// Framebuffer width, in pixels.
    pub fn width(&self) -> u32 {
        self.texture.size.width
    }

    /// Framebuffer height, in pixels.
    pub fn height(&self) -> u32 {
        self.texture.size.height
    }
}

impl RenderTarget for Framebuffer {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

impl Bind for Framebuffer {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.texture.view),
        }
    }
}

impl Canvas for Framebuffer {
    type Color = Bgra8;

    fn clear(&self, color: Self::Color, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self.texture, color, device, encoder);
        Texture::clear(&self.depth.texture, 0f32, device, encoder);
    }

    fn fill(&self, buf: &[Self::Color], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self.texture, buf, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[Self::Color],
        rect: SizedRect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self.texture, buf, rect, device, encoder);
    }

    fn blit(
        &self,
        from: SizedRect<u32, ScreenSpace>,
        dst: SizedRect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::blit(&self.texture, from, dst, encoder);
    }
}
