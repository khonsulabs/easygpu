use crate::prelude::BindingGroup;
use crate::renderer::{Draw, RenderPassExt};

#[derive(Debug)]
pub struct VertexBuffer {
    pub size: u32,
    pub wgpu: wgpu::Buffer,
}

impl Draw for VertexBuffer {
    fn draw<'a>(&'a self, binding: &'a BindingGroup, pass: &mut wgpu::RenderPass<'a>) {
        // TODO: If we attempt to draw more vertices than exist in the buffer, because
        // 'size' was guessed wrong, we get a wgpu error. We should somehow try to
        // get the pipeline layout to know here if the buffer we're trying to draw
        // is the right size. Another option is to create buffers from the pipeline,
        // so that we can check at creation time whether the data passed in matches
        // the format.
        pass.set_binding(binding, &[]);
        pass.draw_buffer(self);
    }
}

impl VertexBuffer {
    pub fn slice(&self) -> wgpu::BufferSlice {
        self.wgpu.slice(0..self.size as u64)
    }
}
