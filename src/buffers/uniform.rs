use crate::binding::Bind;

/// A uniform buffer that can be bound in a 'BindingGroup'.
#[derive(Debug)]
pub struct UniformBuffer {
    pub wgpu: wgpu::Buffer,
    pub size: usize,
    pub count: usize,
}

impl Bind for UniformBuffer {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::Buffer(
                self.wgpu
                    .slice(0..(self.size * self.count) as wgpu::BufferAddress),
            ),
        }
    }
}
