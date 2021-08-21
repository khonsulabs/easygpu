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

pub use figures;
pub use wgpu;

pub mod prelude {
    pub use super::{
        binding::*, buffers::*, canvas::*, color::*, device::*, error::*, frame::*, pipeline::*,
        renderable::*, renderer::*, sampler::*, shader::*, texture::*, transform::*, vertex::*,
        wgpu,
    };
}
