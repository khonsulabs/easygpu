use figures::SizedRect;

use crate::{device::Device, transform::ScreenSpace};

pub trait Canvas {
    type Color;

    fn clear(&self, color: Self::Color, device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn fill(&self, buf: &[Self::Color], device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn transfer(
        &self,
        buf: &[Self::Color],
        r: SizedRect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    );
    fn blit(
        &self,
        from: SizedRect<u32, ScreenSpace>,
        dst: SizedRect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    );
}
