//! Defines layout and buffer of our images.
use canvas::layout::Layout;

/// The byte layout of a buffer.
///
/// An inner invariant is that the layout fits in memory and in particular into a `usize`.
#[derive(Clone, PartialEq, Eq)]
pub struct BufferLayout {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bytes_per_texel: usize,
}

/// Describe a row-major rectangular matrix layout.
///
/// This is only concerned with byte-buffer compatibility and not type or color space semantics of
/// texels. It assumes a row-major layout without space between texels of a row as that is the most
/// efficient and common such layout.
pub struct RowLayoutDescription {
    pub width: u32,
    pub height: u32,
    pub stride: u64,
}

pub struct ImageBuffer {
    inner: canvas::Canvas<BufferLayout>,
}

/// Describes an image semantically.
#[derive(Clone, PartialEq)]
pub struct Descriptor {
    /// The byte and physical layout of the buffer.
    pub layout: BufferLayout,
    /// Describe how each single texel is interpreted.
    pub texel: Texel,
}

#[derive(Clone, PartialEq)]
pub struct Texel {
    /// Which part of the image a single texel refers to.
    pub block: Block,
    /// How numbers and channels are encoded into the texel.
    pub samples: Samples,
    /// How the numbers relate to physical quantities, important for conversion.
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Block {
    /// Each texel is a single pixel.
    Pixel,
    /// Each texel refers to two pixels across width.
    Sub1x2,
    /// Each texel refers to four pixels across width.
    Sub1x4,
    /// Each texel refers to a two-by-two block.
    Sub2x2,
    /// Each texel refers to a two-by-four block.
    Sub2x4,
    /// Each texel refers to a four-by-four block.
    Sub4x4,
}

/// The bit encoding of values within the texel bytes.
#[derive(Clone, PartialEq)]
pub struct Samples {
    /// Which values are encoded, which controls the applicable color spaces.
    pub parts: SampleParts,
    /// How the values are encoded as bits in the bytes.
    pub bits: SampleBits,
}

/// Describes which values are present in a texel.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SampleParts {
    A,
    R,
    G,
    B,
    Rgb,
    Bgr,
    Rgba,
    Rgbx,
    Bgra,
    Bgrx,
    Argb,
    Xrgb,
    Abgr,
    Xbgr,
    Yuv,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SampleBits {
    /// A single 8-bit integer.
    Int8,
    /// Three packed integer.
    Int332,
    /// Three packed integer.
    Int233,
    /// Four packed integer.
    Int4x4,
    /// Four packed integer, one component ignored.
    Inti444,
    /// Four packed integer, one component ignored.
    Int444i,
    /// Three packed integer.
    Int565,
    /// Three 8-bit integer.
    Int8x3,
    /// Four 8-bit integer.
    Int8x4,
    /// Four packed integer.
    Int1010102,
    /// Four packed integer.
    Int2101010,
    /// Three packed integer, one component ignored.
    Int101010i,
    /// Three packed integer, one component ignored.
    Inti101010,
    /// Four half-floats.
    Float16x4,
    /// Four floats.
    Float32x4,
}

/// Describes a single channel from an image.
/// Note that it must match the descriptor when used in `extract` and `inject`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorChannel {
    R,
    G,
    B,
}

#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub enum Color {
    /// A common model based on the CIE 1931 XYZ observer.
    Xyz {
        primary: Primaries,
        transfer: Transfer,
        whitepoint: Whitepoint,
        luminance: Luminance,
    },
}

/// Transfer functions from encoded chromatic samples to physical quantity.
///
/// Ignoring viewing environmental effects, this describes a pair of functions that are each others
/// inverse: An electro-optical transfer (EOTF) and opto-electronic transfer function (OETF) that
/// describes how scene lighting is encoded as an electric signal. These are applied to each
/// stimulus value.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Transfer {
    Bt709,
    Bt470M,
    Bt601,
    Smpte240,
    Linear,
    Srgb,
    Bt2020_10bit,
    Bt2020_12bit,
    Smpte2084,
    /// Another name for Smpte2084.
    Bt2100Pq,
    Bt2100Hlg,
}

/// The reference brightness of the color specification.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Luminance {
    /// 100cd/m².
    Sdr,
    /// 10_000cd/m².
    /// Known as high-dynamic range.
    Hdr,
}

/// The relative stimuli of the three corners of a triangular gamut.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Primaries {
    Bt601_525,
    Bt601_625,
    Bt709,
    Smpte240,
    Bt2020,
    Bt2100,
}

/// The whitepoint/standard illuminant.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Whitepoint {
    D65,
}

impl Descriptor {
    /// Get the texel describing a single channel.
    /// Returns None if the channel is not contained, or if it can not be extracted on its own.
    pub fn channel_texel(&self, channel: ColorChannel) -> Option<Texel> {
        self.texel.channel_texel(channel)
    }
}

impl Texel {
    /// Get the texel describing a single channel.
    /// Returns None if the channel is not contained, or if it can not be extracted on its own.
    pub fn channel_texel(&self, channel: ColorChannel) -> Option<Texel> {
        use Block::*;
        use SampleParts::*;
        use SampleBits::*;
        let parts = match self.samples.parts {
            Rgb | Rgbx | Rgba | Bgrx | Bgra | Abgr | Argb | Xrgb | Xbgr => match channel {
                ColorChannel::R => R,
                ColorChannel::G => G,
                ColorChannel::B => B,
                _ => return None,
            },
            _ => return None,
        };
        let bits = match self.samples.bits {
            Int8 | Int8x3 | Int8x4 => Int8,
            _ => return None,
        };
        let block = match self.block {
            Pixel | Sub1x2 | Sub1x4 | Sub2x2 | Sub2x4 | Sub4x4 => self.block,
            _ => return None,
        };
        Some(Texel {
            samples: Samples {
                bits,
                parts
            },
            block,
            color: self.color.clone(),
        })
    }
}

impl ImageBuffer {
    pub fn layout(&self) -> &BufferLayout {
        self.inner.layout()
    }
}

impl BufferLayout {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Layout for BufferLayout {
    fn byte_len(&self) -> usize {
        // No overflow due to inner invariant.
        (self.width as usize) * (self.height as usize) * (self.bytes_per_texel as usize)
    }
}

impl From<image::DynamicImage> for ImageBuffer {
    fn from(image: image::DynamicImage) -> ImageBuffer {
        use image::GenericImageView;
        let (width, height) = image.dimensions();

        let layout = BufferLayout {
            width,
            height,
            bytes_per_texel: if image.as_flat_samples_u8().is_some() {
                1
            } else if image.as_flat_samples_u16().is_some() {
                2
            } else {
                unreachable!("");
            },
        };

        let inner = canvas::Canvas::with_bytes(layout, image.as_bytes());
        ImageBuffer { inner }
    }
}
