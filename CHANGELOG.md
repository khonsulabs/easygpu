# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.0

### Changes

- `wgpu` has been updated to 0.12.

## v0.0.15

### Changes

- `BindingType::SampledTexture` now specifies filterable as `true`. This caused
  issues when upgrading to wgpu 0.11.1.

## v0.0.14

### Changes

- `wgpu` has been updated to 0.11. There are no public API changes for this
  update.

## v0.0.13

### Breaking API Changes

- `wgpu` has been updated to 0.10. With it come these breaking API changes:
  - `SwapChain` has been removed. Change existing calls from `create_swap_chain` to `configure`.
  - `Renderer` has a new method, `current_frame` which returns a `RenderFrame`. This can be used in place of the former `SwapChainTexture` type.
- `easygpu` now uses `figures` for its math types, and `ScreenScale` and
  `WorldScale` now are type aliases for the unit types provided by that crate.
  If you're using functionality that was in `euclid` but is no longer available
  in `figures`, please submit [an
  issue](https://github.com/khonsulabs/figures/issues). We may not add all
  requested functionality, but as long as it extends one of the types `figures`
  already has, it likely will be added upon request.

  One of the truly breaking changes is `ScreenTransformation::ortho()`. The
  parameter order is now `left`, `top`, `right`, `bottom`, `near`, `far`. This
  order matches my personal preference of the order in which sides of a
  rectangle should be specified. Personal preference, but since it was getting
  reimplemented, I made this change.
- Basic MSAA support has been added. Multisampling textures seems like its own ball of wax. This is aimed at allowing rendering to use multisampling to produce antialiasing around geometry borders. The lyon examples have been updated to use 4 samples.
  - `Renderer::for_surface`, `Renderer::offscreen`, `Device::create_texture`, `Device::create_framebuffer`, `Device::create_zbuffer` now take `sample_count` as a parameter. Pass in 1 to disable MSAA rendering. Any higher number controls the number of samples taken.
  - `Device::create_pipeline` now accepts a `MultisampleState` parameter to control multisampling.
  - `Renderer::texture` now takes an additional argument to specify if the texture should be multisampled. If true, the texture will be created using the sample count the renderer was initialized with.
  - `Frame::pass` now accepts a multisample_buffer parameter. This is a texture view for a multisampled texture that will be used to enable MSAA rendering. If None is passed, MSAA will not take place.
- `easygpu-lyon` has now updated to lyon 0.17.

### Changed

- `easygpu-lyon` is now maintained as part of this repository. The version
  numbers were already tightly linked. This changelog will now contain
  information relating to both.

## v0.0.12

### Changed

- Updated to `wgpu` 0.9. No API changes were necessary.

## v0.0.11

### Breaking API Changes

- `Device::new()` and `Renderer::new()` have been renamed to `for_surface()`.
- `Renderer::create_texture()` and `Device::create_texture()` now require you to specify the `TextureUsage`. Previously, `wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST` was hard-coded.

### Added

- `Device::offscreen()` and `Renderer::offscreen()` have been added to enable offscren rendering. Offscreen types will panic if used with any APIs that require an active wgpu surface, such as `create_swap_chain()`.
