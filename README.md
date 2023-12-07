# easygpu

> *This crate is unmaintained*. A rewrite of [Kludgine][kludgine] has embraced
> an API design supporting [encapsulation][encapsulating]. When this rewrite was
> occuring, the lines between what should belong in easygpu and what should be
> in Kludgine were hard to define, so the new version of Kludgine no longer is
> based on this crate.
>
> If someone reading this wishes to take over maintenance and updates, please
> mention [@ecton](https://github.com/ecton) in an issue on your fork of this
> project that has been updated to the current wgpu version, and I will transfer
> publishing rights on crates.io to you.

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
# Only needed if using the 2018 edition
resolver = "2"

[dependencies]
easygpu = "0.1.0"
```

## MIT License

As with most code from [Khonsu Labs](https://khonsulabs.com), this repository is open source under the [MIT License](./LICENSE.txt)

[encapsulating]: https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work
[kludgine]: https://github.com/khonsulabs/kludgine
