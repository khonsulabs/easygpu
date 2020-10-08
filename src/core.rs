use crate::{
    color::{Bgra8, Rgba, Rgba8},
    error::Error,
    transform::ScreenSpace,
};
use euclid::{Box2D, Point2D, Rect, Size2D};
use raw_window_handle::HasRawWindowHandle;
use std::ops::Range;

pub trait Renderable {
    fn buffer(&self, r: &Renderer) -> VertexBuffer;

    fn finish(self, r: &Renderer) -> VertexBuffer
    where
        Self: std::marker::Sized,
    {
        self.buffer(r)
    }
}

impl Rgba {
    fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

pub trait Draw {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass);
}

/// A GPU Shader.
#[derive(Debug)]
pub struct Shader {
    module: wgpu::ShaderModule,
}

/// Shader stage.
#[derive(Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    fn to_wgpu(&self) -> wgpu::ShaderStage {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStage::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStage::FRAGMENT,
        }
    }
}

pub trait Canvas {
    type Color;

    fn clear(&self, color: Self::Color, device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn fill(&self, buf: &[Self::Color], device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn transfer(
        &self,
        buf: &[Self::Color],
        w: u32,
        h: u32,
        r: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    );
    fn blit(
        &self,
        from: Rect<f32, ScreenSpace>,
        dst: Rect<f32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    );
}

/// A group of bindings.
#[derive(Debug)]
pub struct BindingGroup {
    wgpu: wgpu::BindGroup,
    set_index: u32,
}

impl BindingGroup {
    fn new(set_index: u32, wgpu: wgpu::BindGroup) -> Self {
        Self { set_index, wgpu }
    }
}

/// The layout of a `BindingGroup`.
#[derive(Debug)]
pub struct BindingGroupLayout {
    wgpu: wgpu::BindGroupLayout,
    size: usize,
    set_index: u32,
}

impl BindingGroupLayout {
    fn new(set_index: u32, layout: wgpu::BindGroupLayout, size: usize) -> Self {
        Self {
            wgpu: layout,
            size,
            set_index,
        }
    }
}

/// A trait representing a resource that can be bound.
pub trait Bind {
    fn binding(&self, index: u32) -> wgpu::Binding;
}

/// A uniform buffer that can be bound in a 'BindingGroup'.
#[derive(Debug)]
pub struct UniformBuffer {
    wgpu: wgpu::Buffer,
    size: usize,
    count: usize,
}

impl Bind for UniformBuffer {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Buffer {
                buffer: &self.wgpu,
                range: 0..(self.size as wgpu::BufferAddress),
            },
        }
    }
}

/// Z-Depth buffer.
#[derive(Debug)]
pub struct ZBuffer {
    pub texture: Texture,
}

impl ZBuffer {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
}

/// Off-screen framebuffer. Can be used as a render target in render passes.
#[derive(Debug)]
pub struct Framebuffer {
    pub texture: Texture,
    pub depth: ZBuffer,
}

impl Framebuffer {
    /// Size in pixels of the framebuffer.
    pub fn size(&self) -> usize {
        self.texture.size.cast::<usize>().area()
    }

    /// Framebuffer width, in pixels.
    pub fn width(&self) -> u32 {
        self.texture.size.width
    }

    /// Framebuffer height, in pixels.
    pub fn height(&self) -> u32 {
        self.texture.size.height
    }
}

impl RenderTarget for Framebuffer {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

impl Bind for Framebuffer {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.texture.view),
        }
    }
}

impl Canvas for Framebuffer {
    type Color = Bgra8;

    fn clear(&self, color: Bgra8, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self.texture, color, device, encoder);
        Texture::clear(&self.depth.texture, 0f32, device, encoder);
    }

    fn fill(&self, buf: &[Bgra8], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self.texture, buf, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[Bgra8],
        w: u32,
        h: u32,
        rect: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self.texture, buf, w, h, rect, device, encoder);
    }

    fn blit(
        &self,
        from: Rect<f32, ScreenSpace>,
        dst: Rect<f32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::blit(&self.texture, from, dst, encoder);
    }
}

#[derive(Debug)]
pub struct Texture {
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,
    extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,

    pub size: Size2D<u32, ScreenSpace>,
}

impl Texture {
    pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    fn clear<T>(
        texture: &Texture,
        value: T,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Clone,
    {
        let capacity = texture.size.cast::<usize>().area();
        let mut texels: Vec<T> = Vec::with_capacity(capacity);
        texels.resize(capacity, value);

        let (head, body, tail) = unsafe { texels.align_to::<Rgba8>() };
        assert!(head.is_empty());
        assert!(tail.is_empty());

        Self::fill(texture, body, device, encoder);
    }

    fn fill<T: 'static>(
        texture: &Texture,
        texels: &[T],
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Clone + Copy,
    {
        assert_eq!(
            texels.len() as u32,
            texture.size.area(),
            "fatal: incorrect length for texel buffer"
        );

        let buf = device
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&texels);

        Self::copy(
            &texture.wgpu,
            Point2D::default(),
            texture.size,
            texture.extent,
            &buf,
            encoder,
        );
    }

    fn transfer<T: 'static>(
        texture: &Texture,
        texels: &[T],
        width: u32,
        height: u32,
        rect: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Into<Rgba8> + Clone + Copy,
    {
        // Wgpu's coordinate system has a downwards pointing Y axis.
        let destination = rect.to_box2d();
        // Make sure we have a positive rectangle
        let destination = Box2D::<i32, ScreenSpace>::new(
            Point2D::new(
                destination.min.x.min(destination.max.x),
                destination.min.y.min(destination.max.y),
            ),
            Point2D::new(
                destination.max.x.max(destination.min.x),
                destination.max.y.max(destination.min.y),
            ),
        );
        // flip y, making it negative in the y direction
        let destination = Box2D::new(
            Point2D::new(destination.min.x, destination.max.y),
            Point2D::new(destination.max.x, destination.min.y),
        );
        let rect = destination.to_rect();

        // The width and height of the transfer area.
        let destination_size = rect.size.abs().cast::<u32>();

        // The destination coordinate of the transfer, on the texture.
        // We have to invert the Y coordinate as explained above.
        let destination_point = Point2D::new(
            rect.origin.x as f32,
            texture.size.height as f32 - rect.origin.y as f32,
        );

        assert_eq!(
            texels.len() as u32,
            width * height,
            "fatal: incorrect length for texel buffer"
        );
        assert!(
            destination_size.area() <= texture.size.area(),
            "fatal: transfer size must be <= texture size"
        );

        let buf = device
            .device
            .create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&texels);

        let extent = wgpu::Extent3d {
            width: destination_size.width,
            height: destination_size.height,
            depth: 1,
        };
        Self::copy(
            &texture.wgpu,
            destination_point,
            destination_size,
            extent,
            &buf,
            encoder,
        );
    }

    fn blit(
        &self,
        src: Rect<f32, ScreenSpace>,
        dst: Rect<f32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        assert!(
            (src.width() - dst.width()).abs() <= f32::EPSILON,
            "source and destination rectangles must be of the same size"
        );
        assert!(
            (src.height() - dst.height()).abs() <= f32::EPSILON,
            "source and destination rectangles must be of the same size"
        );

        encoder.copy_texture_to_texture(
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: src.origin.x,
                    y: src.origin.y,
                    z: 0.0,
                },
            },
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: dst.origin.x,
                    y: dst.origin.y,
                    z: 0.0,
                },
            },
            wgpu::Extent3d {
                width: src.width() as u32,
                height: src.height() as u32,
                depth: 1,
            },
        );
    }

    fn copy(
        texture: &wgpu::Texture,
        origin: Point2D<f32, ScreenSpace>,
        size: Size2D<u32, ScreenSpace>,
        extent: wgpu::Extent3d,
        buffer: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                offset: 0,
                row_pitch: 4 * size.width,
                image_height: size.height,
            },
            wgpu::TextureCopyView {
                texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: origin.x,
                    y: origin.y,
                    z: 0.0,
                },
            },
            extent,
        );
    }
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

impl Canvas for Texture {
    type Color = Rgba8;

    fn fill(&self, buf: &[Rgba8], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(&self, buf, device, encoder);
    }

    fn clear(&self, color: Rgba8, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(&self, color, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[Rgba8],
        w: u32,
        h: u32,
        rect: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self, buf, w, h, rect, device, encoder);
    }

    fn blit(
        &self,
        src: Rect<f32, ScreenSpace>,
        dst: Rect<f32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::blit(&self, src, dst, encoder);
    }
}

impl From<Framebuffer> for Texture {
    fn from(fb: Framebuffer) -> Self {
        fb.texture
    }
}

#[derive(Debug)]
pub struct Sampler {
    wgpu: wgpu::Sampler,
}

impl Bind for Sampler {
    fn binding(&self, index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: index as u32,
            resource: wgpu::BindingResource::Sampler(&self.wgpu),
        }
    }
}

#[derive(Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

impl Filter {
    fn to_wgpu(&self) -> wgpu::FilterMode {
        match self {
            Filter::Nearest => wgpu::FilterMode::Nearest,
            Filter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

#[derive(Debug)]
pub struct VertexBuffer {
    pub size: u32,
    wgpu: wgpu::Buffer,
}

impl Draw for VertexBuffer {
    fn draw(&self, binding: &BindingGroup, pass: &mut Pass) {
        // TODO: If we attempt to draw more vertices than exist in the buffer, because
        // 'size' was guessed wrong, we get a wgpu error. We should somehow try to
        // get the pipeline layout to know here if the buffer we're trying to draw
        // is the right size. Another option is to create buffers from the pipeline,
        // so that we can check at creation time whether the data passed in matches
        // the format.
        pass.set_binding(binding, &[]);
        pass.draw_buffer(&self);
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    wgpu: wgpu::Buffer,
}

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
            VertexFormat::Float => wgpu::VertexFormat::Float,
            VertexFormat::Float2 => wgpu::VertexFormat::Float2,
            VertexFormat::Float3 => wgpu::VertexFormat::Float3,
            VertexFormat::Float4 => wgpu::VertexFormat::Float4,
            VertexFormat::UByte4 => wgpu::VertexFormat::Uchar4Norm,
        }
    }
}

/// Describes a 'VertexBuffer' layout.
#[derive(Default, Debug)]
pub struct VertexLayout {
    wgpu_attrs: Vec<wgpu::VertexAttributeDescriptor>,
    size: usize,
}

impl VertexLayout {
    pub fn from(formats: &[VertexFormat]) -> Self {
        let mut vl = Self::default();
        for vf in formats {
            vl.wgpu_attrs.push(wgpu::VertexAttributeDescriptor {
                shader_location: vl.wgpu_attrs.len() as u32,
                offset: vl.size as wgpu::BufferAddress,
                format: vf.to_wgpu(),
            });
            vl.size += vf.bytesize();
        }
        vl
    }

    fn to_wgpu(&self) -> wgpu::VertexBufferDescriptor {
        wgpu::VertexBufferDescriptor {
            stride: self.size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: self.wgpu_attrs.as_slice(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline Bindings
///////////////////////////////////////////////////////////////////////////////

/// A binding type.
#[derive(Debug)]
pub enum BindingType {
    UniformBuffer,
    UniformBufferDynamic,
    Sampler,
    SampledTexture,
}

impl BindingType {
    fn to_wgpu(&self) -> wgpu::BindingType {
        match self {
            BindingType::UniformBuffer => wgpu::BindingType::UniformBuffer { dynamic: false },
            BindingType::UniformBufferDynamic => wgpu::BindingType::UniformBuffer { dynamic: true },
            BindingType::SampledTexture => wgpu::BindingType::SampledTexture {
                multisampled: false,
                dimension: wgpu::TextureViewDimension::D2,
            },
            BindingType::Sampler => wgpu::BindingType::Sampler,
        }
    }
}

#[derive(Debug)]
pub struct Binding {
    pub binding: BindingType,
    pub stage: ShaderStage,
}

///////////////////////////////////////////////////////////////////////////////
/// Pipeline
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Pipeline {
    wgpu: wgpu::RenderPipeline,

    pub layout: PipelineLayout,
    pub vertex_layout: VertexLayout,
}

impl<'a> AbstractPipeline<'a> for Pipeline {
    type PrepareContext = ();
    type Uniforms = ();

    fn description() -> PipelineDescription<'a> {
        PipelineDescription {
            vertex_layout: &[],
            pipeline_layout: &[],
            vertex_shader: &[],
            fragment_shader: &[],
        }
    }

    fn setup(pipeline: Self, _dev: &Device) -> Self {
        pipeline
    }

    fn apply(&self, pass: &mut Pass) {
        pass.wgpu.set_pipeline(&self.wgpu);
    }

    fn prepare(&'a self, _unused: ()) -> Option<(&'a UniformBuffer, Vec<()>)> {
        None
    }
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

    fn to_wgpu(&self) -> (wgpu::BlendFactor, wgpu::BlendFactor, wgpu::BlendOperation) {
        (
            self.src_factor.to_wgpu(),
            self.dst_factor.to_wgpu(),
            self.operation.to_wgpu(),
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
    fn to_wgpu(&self) -> wgpu::BlendFactor {
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
    fn to_wgpu(&self) -> wgpu::BlendOperation {
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

pub trait AbstractPipeline<'a> {
    type PrepareContext;
    type Uniforms: Copy + 'static;

    fn description() -> PipelineDescription<'a>;
    fn setup(pip: Pipeline, dev: &Device) -> Self;
    fn apply(&self, pass: &mut Pass);
    fn prepare(
        &'a self,
        t: Self::PrepareContext,
    ) -> Option<(&'a UniformBuffer, Vec<Self::Uniforms>)>;
}

#[derive(Debug)]
pub struct PipelineDescription<'a> {
    pub vertex_layout: &'a [VertexFormat],
    pub pipeline_layout: &'a [Set<'a>],
    pub vertex_shader: &'static [u8],
    pub fragment_shader: &'static [u8],
}

///////////////////////////////////////////////////////////////////////////////
/// Frame
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Frame {
    encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn new(encoder: wgpu::CommandEncoder) -> Self {
        Self { encoder }
    }

    pub fn pass<T: RenderTarget>(&mut self, op: PassOp, view: &T) -> Pass {
        Pass::begin(
            &mut self.encoder,
            &view.color_target(),
            &view.zdepth_target(),
            op,
        )
    }

    pub fn copy(&mut self, src: &UniformBuffer, dst: &UniformBuffer) {
        self.encoder.copy_buffer_to_buffer(
            &src.wgpu,
            0,
            &dst.wgpu,
            0,
            (src.size * src.count) as wgpu::BufferAddress,
        );
    }

    pub fn encoder(&self) -> &wgpu::CommandEncoder {
        &self.encoder
    }

    pub fn encoder_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Pass
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Pass<'a> {
    wgpu: wgpu::RenderPass<'a>,
}

impl<'a> Pass<'a> {
    pub fn begin(
        encoder: &'a mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        op: PassOp,
    ) -> Self {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &view,
                load_op: op.to_wgpu(),
                store_op: wgpu::StoreOp::Store,
                clear_color: match op {
                    PassOp::Clear(color) => color.to_wgpu(),
                    PassOp::Load() => Rgba::TRANSPARENT.to_wgpu(),
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: depth,
                depth_load_op: op.to_wgpu(),
                depth_store_op: wgpu::StoreOp::Store,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                clear_stencil: 0,
            }),
        });
        Pass { wgpu: pass }
    }
    pub fn set_pipeline<T>(&mut self, pipeline: &T)
    where
        T: AbstractPipeline<'a>,
    {
        pipeline.apply(self);
    }
    pub fn set_binding(&mut self, group: &BindingGroup, offsets: &[u64]) {
        self.wgpu
            .set_bind_group(group.set_index, &group.wgpu, offsets);
    }
    pub fn set_index_buffer(&mut self, index_buf: &IndexBuffer) {
        self.wgpu.set_index_buffer(&index_buf.wgpu, 0)
    }
    pub fn set_vertex_buffer(&mut self, vertex_buf: &VertexBuffer) {
        self.wgpu.set_vertex_buffers(0, &[(&vertex_buf.wgpu, 0)])
    }
    pub fn draw<T: Draw>(&mut self, drawable: &T, binding: &BindingGroup) {
        drawable.draw(binding, self);
    }
    pub fn draw_buffer(&mut self, buf: &VertexBuffer) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(0..buf.size, 0..1);
    }
    pub fn draw_buffer_range(&mut self, buf: &VertexBuffer, range: Range<u32>) {
        self.set_vertex_buffer(buf);
        self.wgpu.draw(range, 0..1);
    }
    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.wgpu.draw_indexed(indices, 0, instances)
    }
}

#[derive(Debug)]
pub enum PassOp {
    Clear(Rgba),
    Load(),
}

impl PassOp {
    fn to_wgpu(&self) -> wgpu::LoadOp {
        match self {
            PassOp::Clear(_) => wgpu::LoadOp::Clear,
            PassOp::Load() => wgpu::LoadOp::Load,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// SwapChain & RenderTarget
///////////////////////////////////////////////////////////////////////////////

/// Can be rendered to in a pass.
pub trait RenderTarget {
    /// Color component.
    fn color_target(&self) -> &wgpu::TextureView;
    /// Depth component.
    fn zdepth_target(&self) -> &wgpu::TextureView;
}

#[derive(Debug)]
pub struct SwapChainTexture<'a> {
    pub size: Size2D<u32, ScreenSpace>,

    wgpu: wgpu::SwapChainOutput<'a>,
    depth: &'a ZBuffer,
}

impl RenderTarget for SwapChainTexture<'_> {
    fn color_target(&self) -> &wgpu::TextureView {
        &self.wgpu.view
    }

    fn zdepth_target(&self) -> &wgpu::TextureView {
        &self.depth.texture.view
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    Vsync,
    NoVsync,
}

impl PresentMode {
    fn to_wgpu(&self) -> wgpu::PresentMode {
        match self {
            PresentMode::Vsync => wgpu::PresentMode::Vsync,
            PresentMode::NoVsync => wgpu::PresentMode::NoVsync,
        }
    }
}

impl Default for PresentMode {
    fn default() -> Self {
        PresentMode::Vsync
    }
}

/// A handle to a swap chain.
///
/// A `SwapChain` represents the image or series of images that will be presented to a [`Renderer`].
/// A `SwapChain` may be created with [`Renderer::swap_chain`].
#[derive(Debug)]
pub struct SwapChain {
    pub size: Size2D<u32, ScreenSpace>,

    depth: ZBuffer,
    wgpu: wgpu::SwapChain,
}

impl SwapChain {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    /// Returns the next texture to be presented by the swapchain for drawing.
    ///
    /// When the [`SwapChainTexture`] returned by this method is dropped, the
    /// swapchain will present the texture to the associated [`Renderer`].
    pub fn next(&mut self) -> SwapChainTexture {
        SwapChainTexture {
            depth: &self.depth,
            wgpu: self.wgpu.get_next_texture(),
            size: self.size,
        }
    }

    /// Get the texture format in use
    pub fn format(&self) -> wgpu::TextureFormat {
        Self::FORMAT
    }

    fn descriptor(size: Size2D<u32, ScreenSpace>, mode: PresentMode) -> wgpu::SwapChainDescriptor {
        wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: Self::FORMAT,
            present_mode: mode.to_wgpu(),
            width: size.width,
            height: size.height,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Renderer
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Renderer {
    pub device: Device,
}

impl Renderer {
    pub fn new<W: HasRawWindowHandle>(window: &W) -> Result<Self, Error> {
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            backends: wgpu::BackendBit::METAL | wgpu::BackendBit::VULKAN,
        })
        .ok_or(Error::NoAdaptersFound)?;

        Ok(Self {
            device: Device::new(&adapter, window),
        })
    }

    pub fn swap_chain(&self, size: Size2D<u32, ScreenSpace>, mode: PresentMode) -> SwapChain {
        SwapChain {
            depth: self.device.create_zbuffer(size),
            wgpu: self.device.create_swap_chain(size, mode),
            size,
        }
    }

    pub fn texture(&self, size: Size2D<u32, ScreenSpace>) -> Texture {
        self.device.create_texture(size)
    }

    pub fn framebuffer(&self, size: Size2D<u32, ScreenSpace>) -> Framebuffer {
        self.device.create_framebuffer(size)
    }

    pub fn zbuffer(&self, size: Size2D<u32, ScreenSpace>) -> ZBuffer {
        self.device.create_zbuffer(size)
    }

    pub fn vertex_buffer<T>(&self, verts: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_buffer(verts)
    }

    pub fn uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        self.device.create_uniform_buffer(buf)
    }

    pub fn binding_group(&self, layout: &BindingGroupLayout, binds: &[&dyn Bind]) -> BindingGroup {
        self.device.create_binding_group(layout, binds)
    }

    pub fn sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        self.device.create_sampler(min_filter, mag_filter)
    }

    pub fn pipeline<T>(&self, blending: Blending) -> T
    where
        T: AbstractPipeline<'static>,
    {
        let desc = T::description();
        let pip_layout = self.device.create_pipeline_layout(desc.pipeline_layout);
        let vertex_layout = VertexLayout::from(desc.vertex_layout);
        let vs =
            self.device
                .create_shader("vertex shader", desc.vertex_shader, ShaderStage::Vertex);
        let fs = self.device.create_shader(
            "fragment shader",
            desc.fragment_shader,
            ShaderStage::Fragment,
        );

        T::setup(
            self.device
                .create_pipeline(pip_layout, vertex_layout, blending, &vs, &fs),
            &self.device,
        )
    }

    pub fn read<F>(&mut self, fb: &Framebuffer, f: F)
    where
        F: 'static + FnOnce(&[Bgra8]),
    {
        let mut encoder = self.device.create_command_encoder();

        let bytesize = 4 * fb.size();
        let dst = self.device.device.create_buffer(&wgpu::BufferDescriptor {
            size: bytesize as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &fb.texture.wgpu,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            wgpu::BufferCopyView {
                buffer: &dst,
                offset: 0,
                // TODO: Must be a multiple of 256
                row_pitch: 4 * fb.texture.size.width,
                image_height: fb.texture.size.height,
            },
            fb.texture.extent,
        );
        self.device.submit(&[encoder.finish()]);

        let mut buffer: Vec<u8> = Vec::with_capacity(bytesize);

        dst.map_read_async(
            0,
            bytesize as u64,
            move |result: wgpu::BufferMapAsyncResult<&[u8]>| match result {
                Ok(ref mapping) => {
                    buffer.extend_from_slice(mapping.data);
                    if buffer.len() == bytesize {
                        let (head, body, tail) = unsafe { buffer.align_to::<Bgra8>() };
                        if !(head.is_empty() && tail.is_empty()) {
                            panic!("Renderer::read: framebuffer is not a valid Bgra8 buffer");
                        }
                        f(body);
                    }
                }
                Err(ref err) => panic!("{:?}", err),
            },
        );
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn update_pipeline<'a, T>(&mut self, pip: &'a T, p: T::PrepareContext, f: &mut Frame)
    where
        T: AbstractPipeline<'a>,
    {
        if let Some((buf, unifs)) = pip.prepare(p) {
            self.device
                .update_uniform_buffer::<T::Uniforms>(unifs.as_slice(), buf, &mut f.encoder);
        }
    }

    pub fn frame(&mut self) -> Frame {
        let encoder = self.device.create_command_encoder();
        Frame::new(encoder)
    }

    pub fn present(&mut self, frame: Frame) {
        self.device.submit(&[frame.encoder.finish()]);
    }

    pub fn submit<T: Copy>(&mut self, commands: &[Op<T>]) {
        let mut encoder = self.device.create_command_encoder();
        for c in commands.iter() {
            c.encode(&mut self.device, &mut encoder);
        }
        self.device.submit(&[encoder.finish()]);
    }
}

pub enum Op<'a, T> {
    Clear(&'a dyn Canvas<Color = T>, T),
    Fill(&'a dyn Canvas<Color = T>, &'a [T]),
    Transfer(
        &'a dyn Canvas<Color = T>,
        &'a [T],
        u32,
        u32,
        Rect<i32, ScreenSpace>,
    ),
    Blit(
        &'a dyn Canvas<Color = T>,
        Rect<f32, ScreenSpace>,
        Rect<f32, ScreenSpace>,
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
            Op::Transfer(f, buf, w, h, rect) => {
                f.transfer(buf, w, h, rect, dev, encoder);
            }
            Op::Blit(f, src, dst) => {
                f.blit(src, dst, encoder);
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Device
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
}

impl Device {
    pub fn new<W: HasRawWindowHandle>(adapter: &wgpu::Adapter, window: &W) -> Self {
        let surface = wgpu::Surface::create(window);
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        Self {
            device,
            queue,
            surface,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.device
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }

    pub fn create_swap_chain(
        &self,
        size: Size2D<u32, ScreenSpace>,
        mode: PresentMode,
    ) -> wgpu::SwapChain {
        let desc = SwapChain::descriptor(size, mode);
        self.device.create_swap_chain(&self.surface, &desc)
    }

    pub fn create_pipeline_layout(&self, ss: &[Set]) -> PipelineLayout {
        let mut sets = Vec::new();
        for (i, s) in ss.iter().enumerate() {
            sets.push(self.create_binding_group_layout(i as u32, s.0))
        }
        PipelineLayout { sets }
    }

    pub fn create_shader(&self, _name: &str, source: &[u8], _stage: ShaderStage) -> Shader {
        let buf = std::io::Cursor::new(source);
        let spv = wgpu::read_spirv(buf).unwrap();

        Shader {
            module: self.device.create_shader_module(spv.as_slice()),
        }
    }

    pub fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 })
    }

    pub fn create_texture(&self, size: Size2D<u32, ScreenSpace>) -> Texture {
        let format = Texture::COLOR_FORMAT;
        let texture_extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();

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
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = texture.create_default_view();

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

    pub fn create_zbuffer(&self, size: Size2D<u32, ScreenSpace>) -> ZBuffer {
        let format = ZBuffer::FORMAT;
        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };
        let wgpu = self.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let view = wgpu.create_default_view();

        ZBuffer {
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
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout.wgpu,
                bindings: bindings.as_slice(),
            }),
        )
    }

    pub fn create_buffer<T>(&self, vertices: &[T]) -> VertexBuffer
    where
        T: 'static + Copy,
    {
        VertexBuffer {
            wgpu: self
                .device
                .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
                .fill_from_slice(vertices),
            size: vertices.len() as u32,
        }
    }

    pub fn create_uniform_buffer<T>(&self, buf: &[T]) -> UniformBuffer
    where
        T: 'static + Copy,
    {
        UniformBuffer {
            size: std::mem::size_of::<T>(),
            count: buf.len(),
            wgpu: self
                .device
                .create_buffer_mapped::<T>(
                    buf.len(),
                    wgpu::BufferUsage::UNIFORM
                        | wgpu::BufferUsage::COPY_DST
                        | wgpu::BufferUsage::COPY_SRC,
                )
                .fill_from_slice(buf),
        }
    }

    pub fn create_index(&self, indices: &[u16]) -> IndexBuffer {
        let index_buf = self
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(indices);
        IndexBuffer { wgpu: index_buf }
    }

    pub fn create_sampler(&self, min_filter: Filter, mag_filter: Filter) -> Sampler {
        Sampler {
            wgpu: self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: mag_filter.to_wgpu(),
                min_filter: min_filter.to_wgpu(),
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                compare_function: wgpu::CompareFunction::Always,
            }),
        }
    }

    pub fn create_binding_group_layout(&self, index: u32, slots: &[Binding]) -> BindingGroupLayout {
        let mut bindings = Vec::new();

        for s in slots {
            bindings.push(wgpu::BindGroupLayoutBinding {
                binding: bindings.len() as u32,
                visibility: s.stage.to_wgpu(),
                ty: s.binding.to_wgpu(),
            });
        }
        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: bindings.as_slice(),
            });
        BindingGroupLayout::new(index, layout, bindings.len())
    }

    pub fn update_uniform_buffer<T: Copy + 'static>(
        &self,
        slice: &[T],
        buf: &UniformBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let src = self
            .device
            .create_buffer_mapped::<T>(
                slice.len(),
                wgpu::BufferUsage::UNIFORM
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::MAP_WRITE,
            )
            .fill_from_slice(slice);

        encoder.copy_buffer_to_buffer(
            &src,
            0,
            &buf.wgpu,
            0,
            (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress,
        );
    }

    // MUTABLE API ////////////////////////////////////////////////////////////

    pub fn submit(&mut self, cmds: &[wgpu::CommandBuffer]) {
        self.queue.submit(cmds);
    }

    // PRIVATE API ////////////////////////////////////////////////////////////

    fn create_pipeline(
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
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: sets.as_slice(),
            });

        let (src_factor, dst_factor, operation) = blending.to_wgpu();

        let wgpu = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs.module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs.module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
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
                    format: ZBuffer::FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_read_mask: 0,
                    stencil_write_mask: 0,
                }),
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_attrs],
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
