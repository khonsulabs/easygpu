use crate::binding::Bind;

#[derive(Debug)]
pub struct Sampler {
    pub wgpu: wgpu::Sampler,
}

impl Bind for Sampler {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index as u32,
            resource: wgpu::BindingResource::Sampler(&self.wgpu),
        }
    }
}
