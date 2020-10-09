# easygpu

This crate exists purely as a middle layer for [Kludgine](https://github.com/khonsulabs/kludgine) to interact with wgpu-rs. It was extracted from [rgx](https://github.com/cloudhead/rgx) as part of an attempt to upgrade from wgpu 0.4 to 0.6, which had several breaking issues.

The purpose of this crate is to house some abstractions for wgpu that make life a little easier. For example, Vertex and Index buffers know how big they are. The secondary goal of this crate is to expose how it does all of the easy work, so that if you need to replace parts of it with hand-written WGPU code, you can do it without waiting for this crate to get an update.

## MIT License

As with most code from [Khonsu Labs](https://khonsulabs.com), this repository is open source under the [MIT License](./LICENSE.txt)
