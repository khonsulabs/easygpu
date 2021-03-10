use std::ops::Range;

use crate::{
    binding::{Bind, BindingGroup, BindingGroupLayout},
    buffers::{DepthBuffer, Framebuffer, IndexBuffer, UniformBuffer, VertexBuffer},
    canvas::Canvas,
    color::{Bgra8, Rgba},
    device::Device,
    error::Error,
    frame::Frame,
    pipeline::{AbstractPipeline, Blending},
    sampler::Sampler,
    swapchain::SwapChain,
    texture::Texture,
    transform::ScreenSpace,
    vertex::VertexLayout,
};
use euclid::{Rect, Size2D};
use wgpu::FilterMode;

pub trait Draw {
    fn draw<'a, 'b>(&'a self, binding: &'a BindingGroup, pass: &'b mut wgpu::RenderPass<'a>);
}

#[derive(Debug)]
pub struct Renderer {
    pub device: Device,
}

impl Renderer {
    pub async fn new(surface: wgpu::Surface, instance: &wgpu::Instance) -> Result<Self, Error> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(Error::NoAdaptersFound)?;

        Ok(Self {
            device: Device::new(surface, &adapter).await?,
        })
    }

    pub fn swap_chain<PresentMode: Into<wgpu::PresentMode>>(
        &self,
        size: Size2D<u32, ScreenSpace>,
        mode: PresentMode,
    ) -> SwapChain {
        SwapChain {
            depth: self.device.create_zbuffer(size),
            wgpu: self.device.create_swap_chain(size, mode),
            size,
        }
    }

    pub fn texture(&self, size: Size2D<u32, ScreenSpace>, format: wgpu::TextureFormat) -> Texture {
        self.device.create_texture(size, format)
    }

    pub fn framebuffer(
        &self,
        size: Size2D<u32, ScreenSpace>,
        format: wgpu::TextureFormat,
    ) -> Framebuffer {
        self.device.create_framebuffer(size, format)
    }

    pub fn zbuffer(&self, size: Size2D<u32, ScreenSpace>) -> DepthBuffer {
        self.device.create_zbuffer(size)
    }

    pub fn vertex_buffer<T: bytemuck::Pod>(&self, verts: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_buffer(verts)
    }

    pub fn uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: bytemuck::Pod + 'static + Copy,
    {
        self.device.create_uniform_buffer(buf)
    }

    pub fn binding_group(&self, layout: &BindingGroupLayout, binds: &[&dyn Bind]) -> BindingGroup {
        self.device.create_binding_group(layout, binds)
    }

    pub fn sampler(&self, min_filter: FilterMode, mag_filter: FilterMode) -> Sampler {
        self.device.create_sampler(min_filter, mag_filter)
    }

    pub fn pipeline<T>(&self, blending: Blending) -> T
    where
        T: AbstractPipeline<'static>,
    {
        let desc = T::description();
        let pip_layout = self.device.create_pipeline_layout(desc.pipeline_layout);
        let vertex_layout = VertexLayout::from(desc.vertex_layout);
        let vs = self.device.create_shader(desc.vertex_shader);
        let fs = self.device.create_shader(desc.fragment_shader);

        T::setup(
            self.device.create_pipeline(
                pip_layout,
                vertex_layout,
                blending,
                &vs,
                &fs,
                SwapChain::FORMAT,
            ),
            &self.device,
        )
    }

    pub async fn read<F>(&mut self, fb: &Framebuffer, f: F) -> Result<(), wgpu::BufferAsyncError>
    where
        F: 'static + FnOnce(&[Bgra8]),
    {
        let mut encoder = self.device.create_command_encoder();

        let bytesize = 4 * fb.size();
        let gpu_buffer = self.device.wgpu.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bytesize as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &fb.texture.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            wgpu::BufferCopyView {
                buffer: &gpu_buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    // TODO: Must be a multiple of 256
                    bytes_per_row: 4 * fb.texture.size.width,
                    rows_per_image: fb.texture.size.height,
                },
            },
            fb.texture.extent,
        );
        self.device.submit(vec![encoder.finish()]);

        let mut buffer: Vec<u8> = Vec::with_capacity(bytesize);

        let dst = gpu_buffer.slice(0..bytesize as u64);
        dst.map_async(wgpu::MapMode::Read).await?;

        let view = dst.get_mapped_range();
        buffer.extend_from_slice(&*view);
        if buffer.len() == bytesize {
            let (head, body, tail) = unsafe { buffer.align_to::<Bgra8>() };
            if !(head.is_empty() && tail.is_empty()) {
                panic!("Renderer::read: framebuffer is not a valid Bgra8 buffer");
            }
            f(body);
        }

        gpu_buffer.unmap();

        Ok(())
    }

    pub fn update_pipeline<'a, T>(&mut self, pip: &'a T, p: T::PrepareContext)
    where
        T: AbstractPipeline<'a>,
    {
        if let Some((buffer, uniforms)) = pip.prepare(p) {
            self.device
                .update_uniform_buffer::<T::Uniforms>(uniforms.as_slice(), buffer);
        }
    }

    pub fn frame(&mut self) -> Frame {
        let encoder = self.device.create_command_encoder();
        Frame::new(encoder)
    }

    pub fn present(&mut self, frame: Frame) {
        self.device.submit(vec![frame.encoder.finish()]);
    }

    pub fn submit<T: Copy>(&mut self, commands: &[Op<T>]) {
        let mut encoder = self.device.create_command_encoder();
        for c in commands.iter() {
            c.encode(&mut self.device, &mut encoder);
        }
        self.device.submit(vec![encoder.finish()]);
    }
}

pub enum Op<'a, T> {
    Clear(&'a dyn Canvas<Color = T>, T),
    Fill(&'a dyn Canvas<Color = T>, &'a [T]),
    Transfer {
        f: &'a dyn Canvas<Color = T>,
        buf: &'a [T],
        rect: Rect<i32, ScreenSpace>,
    },
    Blit(
        &'a dyn Canvas<Color = T>,
        Rect<u32, ScreenSpace>,
        Rect<u32, ScreenSpace>,
    ),
}

impl<'a, T> Op<'a, T>
where
    T: Copy,
{
    fn encode(&self, dev: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        match *self {
            Op::Clear(f, color) => {
                f.clear(color, dev, encoder);
            }
            Op::Fill(f, buf) => {
                f.fill(buf, dev, encoder);
            }
            Op::Transfer { f, buf, rect } => {
                f.transfer(buf, rect, dev, encoder);
            }
            Op::Blit(f, src, dst) => {
                f.blit(src, dst, encoder);
            }
        }
    }
}

pub trait RenderPassExt<'a> {
    fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
        depth: &'a wgpu::TextureView,
        op: PassOp,
    ) -> Self;

    fn set_easy_pipeline<'b, T>(&mut self, pipeline: &'a T)
    where
        T: AbstractPipeline<'b>;

    fn set_binding(&mut self, group: &'a BindingGroup, offsets: &[u32]);

    fn set_easy_index_buffer(&mut self, index_buf: &'a IndexBuffer);
    fn set_easy_vertex_buffer(&mut self, vertex_buf: &'a VertexBuffer);
    fn easy_draw<T: Draw>(&mut self, drawable: &'a T, binding: &'a BindingGroup);
    fn draw_buffer(&mut self, buf: &'a VertexBuffer);
    fn draw_buffer_range(&mut self, buf: &'a VertexBuffer, range: Range<u32>);
    fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>);
}

impl<'a> RenderPassExt<'a> for wgpu::RenderPass<'a> {
    fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
        depth: &'a wgpu::TextureView,
        op: PassOp,
    ) -> Self {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: op.to_wgpu(),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        })
    }

    fn set_easy_pipeline<'b, T>(&mut self, pipeline: &'a T)
    where
        T: AbstractPipeline<'b>,
    {
        self.set_pipeline(&pipeline.pipeline.wgpu);
        self.set_binding(&pipeline.bindings, &[]);
    }

    fn set_binding(&mut self, group: &'a BindingGroup, offsets: &[u32]) {
        self.set_bind_group(group.set_index, &group.wgpu, offsets);
    }

    fn set_easy_index_buffer(&mut self, index_buf: &'a IndexBuffer) {
        self.set_index_buffer(index_buf.slice(), wgpu::IndexFormat::Uint16)
    }

    fn set_easy_vertex_buffer(&mut self, vertex_buf: &'a VertexBuffer) {
        self.set_vertex_buffer(0, vertex_buf.slice())
    }

    fn easy_draw<T: Draw>(&mut self, drawable: &'a T, binding: &'a BindingGroup) {
        drawable.draw(binding, self);
    }
    fn draw_buffer(&mut self, buf: &'a VertexBuffer) {
        self.set_easy_vertex_buffer(buf);
        self.draw(0..buf.size, 0..1);
    }
    fn draw_buffer_range(&mut self, buf: &'a VertexBuffer, range: Range<u32>) {
        self.set_easy_vertex_buffer(buf);
        self.draw(range, 0..1);
    }
    fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.draw_indexed(indices, 0, instances)
    }
}

#[derive(Debug)]
pub enum PassOp {
    Clear(Rgba),
    Load(),
}

impl PassOp {
    fn to_wgpu(&self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            PassOp::Clear(color) => wgpu::LoadOp::Clear((*color).into()),
            PassOp::Load() => wgpu::LoadOp::Load,
        }
    }
}

/// Can be rendered to in a pass.
pub trait RenderTarget {
    /// Color component.
    fn color_target(&self) -> &wgpu::TextureView;
    /// Depth component.
    fn zdepth_target(&self) -> &wgpu::TextureView;
}
