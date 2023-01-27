use std::ops::Deref;

use crate::binding::{Binding, BindingGroup, BindingGroupLayout};
use crate::buffers::UniformBuffer;
use crate::device::Device;
use crate::vertex::{VertexFormat, VertexLayout};

#[derive(Debug)]
pub struct Pipeline {
    pub wgpu: wgpu::RenderPipeline,

    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blending {
    src_factor: BlendFactor,
    dst_factor: BlendFactor,
    operation: BlendOp,
}

impl Blending {
    pub fn new(src_factor: BlendFactor, dst_factor: BlendFactor, operation: BlendOp) -> Self {
        Blending {
            src_factor,
            dst_factor,
            operation,
        }
    }

    pub fn constant() -> Self {
        Blending {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOp::Add,
        }
    }

    pub fn as_wgpu(&self) -> (wgpu::BlendFactor, wgpu::BlendFactor, wgpu::BlendOperation) {
        (
            self.src_factor.as_wgpu(),
            self.dst_factor.as_wgpu(),
            self.operation.as_wgpu(),
        )
    }
}

impl Default for Blending {
    fn default() -> Self {
        Blending {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOp::Add,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendFactor {
    One,
    Zero,
    SrcAlpha,
    OneMinusSrcAlpha,
}

impl BlendFactor {
    fn as_wgpu(&self) -> wgpu::BlendFactor {
        match self {
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlendOp {
    Add,
}

impl BlendOp {
    fn as_wgpu(&self) -> wgpu::BlendOperation {
        match self {
            BlendOp::Add => wgpu::BlendOperation::Add,
        }
    }
}

#[derive(Debug)]
pub struct Set<'a>(pub &'a [Binding]);

#[derive(Debug)]
pub struct PipelineLayout {
    pub sets: Vec<BindingGroupLayout>,
}

pub struct PipelineCore {
    pub pipeline: Pipeline,
    pub bindings: BindingGroup,
    pub uniforms: UniformBuffer,
}

pub trait AbstractPipeline<'a>: Deref<Target = PipelineCore> {
    type PrepareContext;
    type Uniforms: bytemuck::Pod + Copy + 'static;

    fn description() -> PipelineDescription<'a>;
    fn setup(pip: Pipeline, dev: &Device) -> Self;
    fn prepare(
        &'a self,
        context: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)>;
}

#[derive(Debug)]
pub struct PipelineDescription<'a> {
    pub vertex_layout: &'a [VertexFormat],
    pub pipeline_layout: &'a [Set<'a>],
    pub vertex_shader: &'static [u8],
    pub fragment_shader: &'static [u8],
}
