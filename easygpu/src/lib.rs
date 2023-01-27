#![deny(clippy::all)]

pub mod binding;
pub mod buffers;
pub mod canvas;
pub mod color;
pub mod device;
pub mod error;
pub mod frame;
pub mod pipeline;
pub mod renderable;
pub mod renderer;
pub mod sampler;
pub mod shader;
pub mod texture;
pub mod transform;
pub mod vertex;

pub use {figures, wgpu};

pub mod prelude {
    pub use super::binding::*;
    pub use super::buffers::*;
    pub use super::canvas::*;
    pub use super::color::*;
    pub use super::device::*;
    pub use super::error::*;
    pub use super::frame::*;
    pub use super::pipeline::*;
    pub use super::renderable::*;
    pub use super::renderer::*;
    pub use super::sampler::*;
    pub use super::shader::*;
    pub use super::texture::*;
    pub use super::transform::*;
    pub use super::vertex::*;
    pub use super::wgpu;
}
