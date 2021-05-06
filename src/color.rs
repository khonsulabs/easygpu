use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Default, Pod, Zeroable)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Given a byte slice, returns a slice of [`Rgba8`] values.
    pub fn align<'a, S: 'a, T: AsRef<[S]> + ?Sized>(bytes: &'a T) -> &'a [Rgba8] {
        let bytes = bytes.as_ref();
        let (head, body, tail) = unsafe { bytes.align_to::<Rgba8>() };

        if !(head.is_empty() && tail.is_empty()) {
            panic!("Rgba8::align: input is not a valid Rgba8 buffer");
        }
        body
    }
}

impl From<Rgba> for Rgba8 {
    fn from(rgba: Rgba) -> Self {
        Self {
            r: (rgba.r * 255.0).round() as u8,
            g: (rgba.g * 255.0).round() as u8,
            b: (rgba.b * 255.0).round() as u8,
            a: (rgba.a * 255.0).round() as u8,
        }
    }
}

impl From<u32> for Rgba8 {
    fn from(rgba: u32) -> Self {
        Self {
            r: (rgba << 24 & 0xFF) as u8,
            g: (rgba << 16 & 0xFF) as u8,
            b: (rgba << 8 & 0xFF) as u8,
            a: (rgba & 0xFF) as u8,
        }
    }
}

/// A BGRA color with 8-bit channels, used when dealing with framebuffers.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Pod, Zeroable)]
pub struct Bgra8 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Bgra8 {
    pub const fn new(b: u8, g: u8, r: u8, a: u8) -> Self {
        Bgra8 { b, g, r, a }
    }

    /// Given a byte slice, returns a slice of `Bgra8` values.
    pub fn align<'a, S: 'a, T: AsRef<[S]> + ?Sized>(bytes: &'a T) -> &'a [Self] {
        let bytes = bytes.as_ref();
        let (head, body, tail) = unsafe { bytes.align_to::<Self>() };

        if !(head.is_empty() && tail.is_empty()) {
            panic!("Bgra8::align: input is not a valid Rgba8 buffer");
        }
        body
    }
}

impl From<u32> for Bgra8 {
    fn from(rgba: u32) -> Self {
        unsafe { std::mem::transmute(rgba) }
    }
}

impl From<Rgba8> for Bgra8 {
    fn from(rgba: Rgba8) -> Self {
        Self {
            b: rgba.b,
            g: rgba.g,
            r: rgba.r,
            a: rgba.a,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Self = Rgba::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Bgra8> for Rgba8 {
    fn from(bgra: Bgra8) -> Rgba8 {
        Rgba8 {
            r: bgra.r,
            g: bgra.g,
            b: bgra.b,
            a: bgra.a,
        }
    }
}

impl From<Rgba8> for Rgba {
    fn from(rgba8: Rgba8) -> Self {
        Self {
            r: (rgba8.r as f32 / 255.0),
            g: (rgba8.g as f32 / 255.0),
            b: (rgba8.b as f32 / 255.0),
            a: (rgba8.a as f32 / 255.0),
        }
    }
}

impl From<Rgba> for wgpu::Color {
    fn from(rgba: Rgba) -> wgpu::Color {
        wgpu::Color {
            r: rgba.r as f64,
            g: rgba.g as f64,
            b: rgba.b as f64,
            a: rgba.a as f64,
        }
    }
}
