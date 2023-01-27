use std::num::NonZeroU32;

use figures::{ExtentsRect, Point, Rectlike, Size, SizedRect};
use wgpu::TextureAspect;

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

    pub size: Size<u32, ScreenSpace>,
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
        let capacity = texture.size.cast::<usize>().area().get();
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
            texels.len() as u32 >= texture.size.area().get(),
            "fatal: incorrect length for texel buffer"
        );

        let buf = device.create_buffer_from_slice(texels, wgpu::BufferUsages::COPY_SRC);

        Self::copy(
            &texture.wgpu,
            SizedRect::new(Point::default(), texture.size),
            texels.len() as u32 / texture.extent.height * 4,
            texture.extent,
            &buf,
            encoder,
        );
    }

    pub fn transfer<T: bytemuck::Pod + 'static>(
        texture: &Texture,
        texels: &[T],
        rect: SizedRect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        T: Into<Rgba8> + Clone + Copy,
    {
        // Wgpu's coordinate system has a downwards pointing Y axis.
        let destination = rect.as_extents();
        // Make sure we have a positive rectangle
        let destination = ExtentsRect::<i32, ScreenSpace>::new(
            Point::new(
                destination.origin.x.min(destination.extent.x),
                destination.origin.y.min(destination.extent.y),
            ),
            Point::new(
                destination.extent.x.max(destination.origin.x),
                destination.extent.y.max(destination.origin.y),
            ),
        );
        // flip y, making it negative in the y direction
        let destination = ExtentsRect::new(
            Point::new(destination.origin.x, destination.extent.y),
            Point::new(destination.extent.x, destination.origin.y),
        );
        let rect = destination.as_sized();

        // The width and height of the transfer area.
        let destination_size = rect.size.abs().cast::<u32>();

        // The destination coordinate of the transfer, on the texture.
        // We have to invert the Y coordinate as explained above.
        let destination_point = Point::new(
            rect.origin.x as u32,
            texture.size.height - rect.origin.y as u32,
        );

        assert!(
            destination_size.area() <= texture.size.area(),
            "fatal: transfer size must be <= texture size"
        );

        let buf = device.create_buffer_from_slice(texels, wgpu::BufferUsages::COPY_SRC);

        let extent = wgpu::Extent3d {
            width: destination_size.width,
            height: destination_size.height,
            depth_or_array_layers: 1,
        };
        Self::copy(
            &texture.wgpu,
            SizedRect::new(destination_point, destination_size),
            texels.len() as u32 / destination_size.height * 4,
            extent,
            &buf,
            encoder,
        );
    }

    fn blit(
        &self,
        src: SizedRect<u32, ScreenSpace>,
        dst: SizedRect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        assert!(
            src.area() != dst.area(),
            "source and destination rectangles must be of the same size"
        );

        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src.origin.x,
                    y: src.origin.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &self.wgpu,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst.origin.x,
                    y: dst.origin.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            wgpu::Extent3d {
                width: src.width().get(),
                height: src.height().get(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn copy(
        texture: &wgpu::Texture,
        destination: SizedRect<u32, ScreenSpace>,
        bytes_per_row: u32,
        extent: wgpu::Extent3d,
        buffer: &wgpu::Buffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(bytes_per_row),
                    rows_per_image: NonZeroU32::new(destination.size.height),
                },
            },
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: destination.origin.x,
                    y: destination.origin.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            extent,
        );
    }
}

impl Bind for Texture {
    fn binding(&self, index: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: index,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

impl Canvas for Texture {
    type Color = Rgba8;

    fn fill(&self, buf: &[Rgba8], device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::fill(self, buf, device, encoder);
    }

    fn clear(&self, color: Rgba8, device: &mut Device, encoder: &mut wgpu::CommandEncoder) {
        Texture::clear(self, color, device, encoder);
    }

    fn transfer(
        &self,
        buf: &[Rgba8],
        rect: SizedRect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::transfer(self, buf, rect, device, encoder);
    }

    fn blit(
        &self,
        src: SizedRect<u32, ScreenSpace>,
        dst: SizedRect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        Texture::blit(self, src, dst, encoder);
    }
}

impl From<Framebuffer> for Texture {
    fn from(fb: Framebuffer) -> Self {
        fb.texture
    }
}
