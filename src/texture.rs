use euclid::{Box2D, Point2D, Rect, Size2D};

use crate::{
    binding::Bind, buffers::Framebuffer, canvas::Canvas, color::Rgba8, device::Device,
    transform::ScreenSpace,
};

#[derive(Debug)]
pub struct Texture {
    pub wgpu: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub extent: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,

    pub size: Size2D<u32, ScreenSpace>,
}

impl Texture {
    pub fn clear<T>(
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

    pub fn fill<T: bytemuck::Pod + 'static>(
        texture: &Texture,
        texels: &[T],
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Clone + Copy,
    {
        assert!(
            texels.len() as u32 >= texture.size.area(),
            "fatal: incorrect length for texel buffer"
        );

        let buf = device.create_buffer_from_slice(texels, wgpu::BufferUsage::COPY_SRC);

        Self::copy(
            &texture.wgpu,
            Rect::new(Point2D::default(), texture.size),
            texels.len() as u32 / texture.extent.height * 4,
            texture.extent,
            &buf,
            encoder,
        );
    }

    pub fn transfer<T: bytemuck::Pod + 'static>(
        texture: &Texture,
        texels: &[T],
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
            rect.origin.x as u32,
            texture.size.height as u32 - rect.origin.y as u32,
        );

        assert!(
            destination_size.area() <= texture.size.area(),
            "fatal: transfer size must be <= texture size"
        );

        let buf = device.create_buffer_from_slice(texels, wgpu::BufferUsage::COPY_SRC);

        let extent = wgpu::Extent3d {
            width: destination_size.width,
            height: destination_size.height,
            depth: 1,
        };
        Self::copy(
            &texture.wgpu,
            Rect::new(destination_point, destination_size),
            texels.len() as u32 / destination_size.height * 4,
            extent,
            &buf,
            encoder,
        );
    }

    fn blit(
        &self,
        src: Rect<u32, ScreenSpace>,
        dst: Rect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        assert!(
            src.area() != dst.area(),
            "source and destination rectangles must be of the same size"
        );

        encoder.copy_texture_to_texture(
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src.origin.x,
                    y: src.origin.y,
                    z: 0,
                },
            },
            wgpu::TextureCopyView {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst.origin.x,
                    y: dst.origin.y,
                    z: 0,
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
        destination: Rect<u32, ScreenSpace>,
        bytes_per_row: u32,
        extent: wgpu::Extent3d,
        buffer: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row,
                    rows_per_image: destination.size.height,
                },
            },
            wgpu::TextureCopyView {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: destination.origin.x,
                    y: destination.origin.y,
                    z: 0,
                },
            },
            extent,
        );
    }
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
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
        rect: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(&self, buf, rect, device, encoder);
    }

    fn blit(
        &self,
        src: Rect<u32, ScreenSpace>,
        dst: Rect<u32, ScreenSpace>,
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
