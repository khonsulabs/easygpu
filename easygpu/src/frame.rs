use wgpu::TextureView;

use crate::{
    buffers::UniformBuffer,
    renderer::{PassOp, RenderPassExt, RenderTarget},
};

#[derive(Debug)]
pub struct Frame {
    pub encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self { encoder }
    }

    pub fn pass<'a>(
        &'a mut self,
        op: PassOp,
        view: &'a impl RenderTarget,
        multisample_buffer: Option<&'a TextureView>,
    ) -> wgpu::RenderPass<'a> {
        let (pass_view, resolve_target) = match multisample_buffer {
            Some(buffer) => (buffer, Some(view.color_target())),
            None => (view.color_target(), None),
        };
        wgpu::RenderPass::begin(
            &mut self.encoder,
            pass_view,
            resolve_target,
            view.zdepth_target(),
            op,
        )
    }

    pub fn copy(&mut self, src: &UniformBuffer, dst: &UniformBuffer) {
        self.encoder.copy_buffer_to_buffer(
            &src.wgpu,
            0,
            &dst.wgpu,
            0,
            (src.size * src.count) as wgpu::BufferAddress,
        );
    }

    pub fn encoder(&self) -> &wgpu::CommandEncoder {
        &self.encoder
    }

    pub fn encoder_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}
