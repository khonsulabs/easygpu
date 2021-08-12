use crate::texture::Texture;

/// Z-Depth buffer.
#[derive(Debug)]
pub struct DepthBuffer {
    pub texture: Texture,
}

impl DepthBuffer {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
}
