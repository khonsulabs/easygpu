#[derive(Debug)]
pub struct Shader {
    pub wgpu: wgpu::ShaderModule,
}

pub use wgpu::ShaderStage;
