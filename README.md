# easygpu

This crate exists purely as a middle layer for [Kludgine](https://github.com/khonsulabs/kludgine) to interact with wgpu-rs. It was extracted from [rgx](https://github.com/cloudhead/rgx) as part of an attempt to upgrade from wgpu 0.4 to 0.6, which had several breaking issues.

The purpose of this crate is to house some abstractions for wgpu that make life a little easier. For example, Vertex and Index buffers know how big they are. The secondary goal of this crate is to expose how it does all of the easy work, so that if you need to replace parts of it with hand-written WGPU code, you can do it without waiting for this crate to get an update.

## Relying on `easygpu`

This project received [its first pull request](https://github.com/khonsulabs/easygpu/pull/1) from another contributor in May of 2021 (thank you!). Despite the warning above, I do think this project can help people who don't want to use Kludgine but are wanting to make `wgpu` a little more approachable.

Because I want that to be possible, I'm going to commit to maintaining a [CHANGELOG](./CHANGELOG.md) which will highight breaking changes. This project is severely under-documented (I will try to address that over time), so this is literally the least I can do to help those using this crate in their projects.

## Getting Started

To use easygpu, your project must be using [the new features
resolver](https://doc.rust-lang.org/cargo/reference/features.html#feature-resolver-version-2). The two
lines to add to your `Cargo.toml` look like this:

```toml
[lib]
resolver = "2"

[dependencies]
easygpu = "0.0.13"
```

The `resolver` requirement is inherited from `wgpu`. This setting [will become
the default in the 2021
edition](https://github.com/rust-lang/cargo/issues/9048).

## MIT License

As with most code from [Khonsu Labs](https://khonsulabs.com), this repository is open source under the [MIT License](./LICENSE.txt)
