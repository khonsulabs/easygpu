use std::marker::PhantomData;
use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use easygpu::prelude::*;
use easygpu::wgpu::TextureFormat;

/// A pipeline for rendering shapes.
pub struct LyonPipeline<T> {
    pipeline: PipelineCore,
    _phantom: PhantomData<T>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
/// The uniforms for the shader.
pub struct Uniforms {
    /// The orthographic projection matrix
    pub ortho: [f32; 16],
    /// The transformation matrix
    pub transform: [f32; 16],
}

pub trait VertexShaderSource {
    fn shader() -> &'static [u8];

    fn sampler_format() -> TextureFormat;
}

pub struct Srgb;
pub struct Normal;

impl VertexShaderSource for Srgb {
    fn shader() -> &'static [u8] {
        include_bytes!("shaders/shape-srgb.vert.spv")
    }

    fn sampler_format() -> TextureFormat {
        TextureFormat::Bgra8UnormSrgb
    }
}

impl VertexShaderSource for Normal {
    fn shader() -> &'static [u8] {
        include_bytes!("shaders/shape.vert.spv")
    }

    fn sampler_format() -> TextureFormat {
        TextureFormat::Bgra8Unorm
    }
}

impl<'a, T> AbstractPipeline<'a> for LyonPipeline<T>
where
    T: VertexShaderSource,
{
    type PrepareContext = ScreenTransformation<f32>;
    type Uniforms = Uniforms;

    fn description() -> PipelineDescription<'a> {
        PipelineDescription {
            vertex_layout: &[VertexFormat::Float3, VertexFormat::UByte4],
            pipeline_layout: &[Set(&[Binding {
                binding: BindingType::UniformBuffer,
                stage: ShaderStages::VERTEX,
            }])],
            vertex_shader: T::shader(),
            fragment_shader: include_bytes!("shaders/shape.frag.spv"),
        }
    }

    fn setup(pipeline: Pipeline, dev: &Device) -> Self {
        let transform = ScreenTransformation::identity().to_array();
        let ortho = ScreenTransformation::identity().to_array();
        let uniforms = dev.create_uniform_buffer(&[self::Uniforms { ortho, transform }]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&uniforms]);

        Self {
            pipeline: PipelineCore {
                pipeline,
                uniforms,
                bindings,
            },
            _phantom: PhantomData::default(),
        }
    }

    fn prepare(
        &'a self,
        ortho: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)> {
        let ortho = ortho.to_array();
        let transform = ScreenTransformation::identity().to_array();
        Some((
            &self.pipeline.uniforms,
            vec![self::Uniforms { transform, ortho }],
        ))
    }
}

impl<T> Deref for LyonPipeline<T> {
    type Target = PipelineCore;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
