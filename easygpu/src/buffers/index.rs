#[derive(Debug)]
pub struct IndexBuffer {
    pub wgpu: wgpu::Buffer,
    pub elements: u32,
}

impl IndexBuffer {
    pub fn slice(&self) -> wgpu::BufferSlice {
        self.wgpu
            .slice(0..(self.elements as usize * std::mem::size_of::<u16>()) as u64)
    }
}
