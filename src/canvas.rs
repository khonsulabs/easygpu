use crate::{device::Device, transform::ScreenSpace};
use euclid::Rect;

pub trait Canvas {
    type Color;

    fn clear(&self, color: Self::Color, device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn fill(&self, buf: &[Self::Color], device: &mut Device, encoder: &mut wgpu::CommandEncoder);
    fn transfer(
        &self,
        buf: &[Self::Color],
        r: Rect<i32, ScreenSpace>,
        device: &mut Device,
        encoder: &mut wgpu::CommandEncoder,
    );
    fn blit(
        &self,
        from: Rect<u32, ScreenSpace>,
        dst: Rect<u32, ScreenSpace>,
        encoder: &mut wgpu::CommandEncoder,
    );
}
