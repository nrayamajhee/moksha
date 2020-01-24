//! Colors
/// u32 <-> Color implementations taken from
/// <https://docs.rs/three/0.4.0/three/color/index.html>

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1. }
    }
    pub fn redish(shade: f32) -> Self {
        Self::rgb(shade, 0., 0.)
    }
    pub fn greenish(shade: f32) -> Self {
        Self::rgb(0., shade, 0.)
    }
    pub fn bluish(shade: f32) -> Self {
        Self::rgb(0., 0., shade)
    }
    pub fn gray(shade: f32) -> Self {
        Self::rgb(shade, shade, shade)
    }
    pub fn black() -> Self {
        Self::gray(0.)
    }
    pub fn white() -> Self {
        Self::gray(1.0)
    }
    pub fn red() -> Self {
        Self::redish(1.0)
    }
    pub fn blue() -> Self {
        Self::bluish(1.0)
    }
    pub fn green() -> Self {
        Self::greenish(1.0)
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl From<[f32; 4]> for Color {
    fn from(c: [f32; 4]) -> Self {
        Self {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        }
    }
}

/// sRGB to Linear
/// Implementation taken from
/// <https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_sRGB_decode.txt>
impl From<u32> for Color {
    fn from(c: u32) -> Self {
        let f = |xu: u32| {
            let x = (xu & 0xFF) as f32 / 255.0;
            if x > 0.04045 {
                ((x + 0.055) / 1.055).powf(2.4)
            } else {
                x / 12.92
            }
        };
        Color::rgb(f(c >> 16), f(c >> 8), f(c))
    }
}

/// Linear to sRGB
/// Implementation taken from <https://en.wikipedia.org/wiki/SRGB>
impl Into<u32> for Color {
    fn into(self) -> u32 {
        let f = |x: f32| -> u32 {
            let y = if x > 0.0031308 {
                let a = 0.055;
                (1.0 + a) * x.powf(-2.4) - a
            } else {
                12.92 * x
            };
            (y * 255.0).round() as u32
        };
        f(self.r) << 16 | f(self.g) << 8 | f(self.b)
    }
}
