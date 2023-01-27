use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use easygpu::buffers::{IndexBuffer, VertexBuffer};
use easygpu::color::Rgba8;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 3],
    pub color: Rgba8,
}

/// Shape is a loaded, prepared ShapeBuilder that is ready to be drawn
pub struct Shape {
    /// Number of indices contained in `indices`
    pub index_count: u32,
    /// The vertices stored in a vertex buffer
    pub vertices: Arc<VertexBuffer>,
    /// An index buffer representing a TriangleList of indices within `vertices`
    pub indices: Arc<IndexBuffer>,
}

impl Shape {
    /// Draws the shape to the Pass.
    ///
    /// You should use `Pass::set_pipeline` before calling this method.
    ///
    /// # Arguments
    ///
    /// * `pass`- The render pass to draw to.
    pub fn draw<'a, 'b>(&'a self, pass: &'b mut easygpu::wgpu::RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertices.slice());
        pass.set_index_buffer(self.indices.slice(), easygpu::wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1)
    }
}
