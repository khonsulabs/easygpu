use euclid::Size2D;

use crate::{buffers::DepthBuffer, renderer::RenderTarget, transform::ScreenSpace};

/// A handle to a swap chain.
///
/// A `SwapChain` represents the image or series of images that will be presented to a [`Renderer`].
/// A `SwapChain` may be created with [`Renderer::swap_chain`].
#[derive(Debug)]
pub struct SwapChain {
    pub wgpu: wgpu::SwapChain,
    pub depth: DepthBuffer,
    pub size: Size2D<u32, ScreenSpace>,
}

impl SwapChain {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    /// Returns the next texture to be presented by the swapchain for drawing.
    ///
    /// When the [`SwapChainTexture`] returned by this method is dropped, the
    /// swapchain will present the texture to the associated [`Renderer`].
    pub fn next_texture(&mut self) -> Result<SwapChainTexture, wgpu::SwapChainError> {
        Ok(SwapChainTexture {
            depth: &self.depth,
            wgpu: self.wgpu.get_current_frame()?,
            size: self.size,
        })
    }

    /// Get the texture format in use
    pub fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    pub fn descriptor<PresentMode: Into<wgpu::PresentMode>>(
        size: Size2D<u32, ScreenSpace>,
        mode: PresentMode,
    ) -> wgpu::SwapChainDescriptor {
        wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: Self::FORMAT,
            present_mode: mode.into(),
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Debug)]
pub struct SwapChainTexture<'a> {
    pub size: Size2D<u32, ScreenSpace>,

    wgpu: wgpu::SwapChainFrame,
    depth: &'a DepthBuffer,
}

impl RenderTarget for SwapChainTexture<'_> {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.wgpu.output.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Vsync,
    NoVsync,
}

impl Into<wgpu::PresentMode> for PresentMode {
    fn into(self) -> wgpu::PresentMode {
        match self {
            PresentMode::Vsync => wgpu::PresentMode::Mailbox,
            PresentMode::NoVsync => wgpu::PresentMode::Immediate,
        }
    }
}

impl Default for PresentMode {
    fn default() -> Self {
        PresentMode::Vsync
    }
}
