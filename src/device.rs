use crate::{
    binding::{Bind, Binding, BindingGroup, BindingGroupLayout},
    buffers::{DepthBuffer, Framebuffer, IndexBuffer, UniformBuffer, VertexBuffer},
    pipeline::{Blending, Pipeline, PipelineLayout, Set},
    sampler::Sampler,
    shader::Shader,
    swapchain::SwapChain,
    texture::Texture,
    transform::ScreenSpace,
    vertex::VertexLayout,
};
use euclid::Size2D;
use raw_window_handle::HasRawWindowHandle;
use wgpu::FilterMode;

#[derive(Debug)]
pub struct Device {
    pub wgpu: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
}

impl Device {
    pub async fn new<W: HasRawWindowHandle>(
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        window: &W,
    ) -> Result<Self, wgpu::RequestDeviceError> {
        let surface = unsafe { instance.create_surface(window) };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: false,
                },
                None,
            )
            .await?;

        Ok(Self {
            wgpu: device,
            queue,
            surface,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.wgpu
    }

    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.wgpu
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.wgpu
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn create_swap_chain<PM: Into<wgpu::PresentMode>>(
        &self,
        size: Size2D<u32, ScreenSpace>,
        mode: PM,
    ) -> wgpu::SwapChain {
        let desc = SwapChain::descriptor(size, mode);
        self.wgpu.create_swap_chain(&self.surface, &desc)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_binding_group_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
    }

    pub fn create_shader(&self, source: &[u8]) -> Shader {
        Shader {
            wgpu: self
                .wgpu
                .create_shader_module(wgpu::util::make_spirv(source)),
        }
    }

    pub fn create_shader_from_wgsl(&self, source: &str) -> Shader {
        Shader {
            wgpu: self
                .wgpu
                .create_shader_module(wgpu::ShaderModuleSource::Wgsl(source.into())),
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.wgpu
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    }

    pub fn create_texture(&self, size: Size2D<u32, ScreenSpace>) -> Texture {
        let format = Texture::COLOR_FORMAT;
        let texture_extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let texture = self.wgpu.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: None,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            wgpu: texture,
            view: texture_view,
            extent: texture_extent,
            format,
            size,
        }
    }

    pub fn create_framebuffer(&self, size: Size2D<u32, ScreenSpace>) -> Framebuffer {
        let format = SwapChain::FORMAT;
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let texture = self.wgpu.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Framebuffer {
            texture: Texture {
                wgpu: texture,
                view,
                extent,
                format,
                size,
            },
            depth: self.create_zbuffer(size),
        }
    }

    pub fn create_zbuffer(&self, size: Size2D<u32, ScreenSpace>) -> DepthBuffer {
        let format = DepthBuffer::FORMAT;
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let wgpu = self.wgpu.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = wgpu.create_view(&wgpu::TextureViewDescriptor::default());

        DepthBuffer {
            texture: Texture {
                wgpu,
                extent,
                view,
                format,
                size,
            },
        }
    }

    pub fn create_binding_group(
        &self,
        layout: &BindingGroupLayout,
        binds: &[&dyn Bind],
    ) -> BindingGroup {
        assert_eq!(
            binds.len(),
            layout.size,
            "layout slot count does not match bindings"
        );

        let mut bindings = Vec::new();

        for (i, b) in binds.iter().enumerate() {
            bindings.push(b.binding(i as u32));
        }

        BindingGroup::new(
            layout.set_index,
            self.wgpu.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout.wgpu,
                label: None,
                entries: bindings.as_slice(),
            }),
        )
    }

    pub fn create_buffer<T>(&self, vertices: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self.create_buffer_from_slice(vertices, wgpu::BufferUsage::VERTEX),
            size: (vertices.len() * std::mem::size_of::<T>()) as u32,
        }
    }

    pub fn create_uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            count: buf.len(),
            wgpu: self.create_buffer_from_slice(
                buf,
                wgpu::BufferUsage::UNIFORM
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC,
            ),
        }
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self.create_buffer_from_slice(indices, wgpu::BufferUsage::INDEX);
        IndexBuffer {
            wgpu: index_buf,
            elements: indices.len() as u32,
        }
    }

    pub fn create_sampler(&self, min_filter: FilterMode, mag_filter: FilterMode) -> Sampler {
        Sampler {
            wgpu: self.wgpu.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter,
                min_filter,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare: Some(wgpu::CompareFunction::Always),
                anisotropy_clamp: None,
                label: None,
            }),
        }
    }

    pub fn create_binding_group_layout(&self, index: u32, slots: &[Binding]) -> BindingGroupLayout {
        let mut bindings = Vec::new();

        for s in slots {
            bindings.push(wgpu::BindGroupLayoutEntry {
                binding: bindings.len() as u32,
                visibility: s.stage,
                ty: s.binding.to_wgpu(),
                count: None,
            });
        }
        let layout = self
            .wgpu
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: bindings.as_slice(),
            });
        BindingGroupLayout::new(index, layout, bindings.len())
    }

    pub fn create_buffer_from_slice<T>(
        &self,
        slice: &[T],
        usage: wgpu::BufferUsage,
    ) -> wgpu::Buffer {
        let byte_length = slice.len() * std::mem::size_of::<T>();
        let src = self.wgpu.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_length as u64,
            mapped_at_creation: true,
            usage,
        });
        let buf_slice = src.slice(0..(byte_length as wgpu::BufferAddress));
        {
            let mut range = buf_slice.get_mapped_range_mut();
            let slice_ptr = slice.as_ptr().cast::<u8>();
            let slice_as_u8 = unsafe { std::slice::from_raw_parts(slice_ptr, byte_length) };
            range.copy_from_slice(slice_as_u8);
        }
        src.unmap();
        src
    }

    pub fn update_uniform_buffer<T: Copy + 'static>(
        &self,
        slice: &[T],
        buf: &UniformBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let src = self.create_buffer_from_slice(
            slice,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &src,
            0,
            &buf.wgpu,
            0,
            (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress,
        );
    }

    pub fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(&mut self, cmds: I) {
        self.queue.submit(cmds);
    }

    pub fn create_pipeline(
        &self,
        pipeline_layout: PipelineLayout,
        vertex_layout: VertexLayout,
        blending: Blending,
        vs: &Shader,
        fs: &Shader,
    ) -> Pipeline {
        let vertex_attrs = vertex_layout.to_wgpu();

        let mut sets = Vec::new();
        for s in pipeline_layout.sets.iter() {
            sets.push(&s.wgpu);
        }
        let layout = &self
            .wgpu
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: sets.as_slice(),
                push_constant_ranges: &[],
            });

        let (src_factor, dst_factor, operation) = blending.to_wgpu();

        let wgpu = self
            .wgpu
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[vertex_attrs],
                },
                layout: Some(layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs.wgpu,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs.wgpu,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: SwapChain::FORMAT,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor,
                        dst_factor,
                        operation,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                    format: DepthBuffer::FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilStateDescriptor {
                        front: wgpu::StencilStateFaceDescriptor::IGNORE,
                        back: wgpu::StencilStateFaceDescriptor::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                }),
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Pipeline {
            layout: pipeline_layout,
            vertex_layout,
            wgpu,
        }
    }
}
