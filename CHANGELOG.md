# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Breaking API Changes

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
