use crate::buffers::VertexBuffer;
use crate::renderer::Renderer;

pub trait Renderable {
    fn buffer(&self, r: &Renderer) -> VertexBuffer;

    fn finish(self, r: &Renderer) -> VertexBuffer
    where
        Self: std::marker::Sized,
    {
        self.buffer(r)
    }
}
