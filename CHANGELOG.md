# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changes

- `easygpu-lyon` is now maintained as part of this repository. The version
  numbers were already tightly linked. This changelog will now contain
  information relating to both.

### Breaking API Changes

- `easygpu` now uses `figures` for its math types, and `ScreenScale` and
  `WorldScale` now are type aliases for the unit types provided by that crate.
  If you're using functionality that was in `euclid` but is no longer available
  in `figures`, please submit [an
  issue](https://github.com/khonsulabs/figures/issues). We may not add all
  requested functionality, but as long as it extends one of the types `figures`
  already has, it likely will be added upon request.

## v0.0.12

### Changed

- Updated to `wgpu` 0.9. No API changes were necessary.

## v0.0.11

### Breaking API Changes

- `Device::new()` and `Renderer::new()` have been renamed to `for_surface()`.
- `Renderer::create_texture()` and `Device::create_texture()` now require you to specify the `TextureUsage`. Previously, `wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST` was hard-coded.

### Added

- `Device::offscreen()` and `Renderer::offscreen()` have been added to enable offscren rendering. Offscreen types will panic if used with any APIs that require an active wgpu surface, such as `create_swap_chain()`.
