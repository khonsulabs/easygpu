#![deny(clippy::all)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]

pub mod color;
pub mod core;
pub mod error;
pub mod renderable;
pub mod transform;

pub use euclid;
pub use wgpu;
