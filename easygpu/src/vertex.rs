#[derive(Debug, Clone, Copy)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    UByte4,
}

impl VertexFormat {
    const fn bytesize(self) -> usize {
        match self {
            VertexFormat::Float => 4,
            VertexFormat::Float2 => 8,
            VertexFormat::Float3 => 12,
            VertexFormat::Float4 => 16,
            VertexFormat::UByte4 => 4,
        }
    }

    const fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            VertexFormat::Float => wgpu::VertexFormat::Float32,
            VertexFormat::Float2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::Float3 => wgpu::VertexFormat::Float32x3,
            VertexFormat::Float4 => wgpu::VertexFormat::Float32x4,
            VertexFormat::UByte4 => wgpu::VertexFormat::Unorm8x4,
        }
    }
}

/// Describes a 'VertexBuffer' layout.
#[derive(Default, Debug)]
pub struct VertexLayout {
    wgpu_attrs: Vec<wgpu::VertexAttribute>,
    size: usize,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.wgpu_attrs.push(wgpu::VertexAttribute {
                shader_location: vl.wgpu_attrs.len() as u32,
                offset: vl.size as wgpu::BufferAddress,
                format: vf.to_wgpu(),
            });
            vl.size += vf.bytesize();
        }
        vl
    }

    pub fn to_wgpu(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: self.wgpu_attrs.as_slice(),
        }
    }
}
