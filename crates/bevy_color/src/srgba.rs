use crate::color_difference::EuclideanDistance;
use crate::oklaba::Oklaba;
use crate::{Alpha, Hsla, LinearRgba, Luminance, Mix, StandardColor};
use bevy_math::Vec4;
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize};
use bevy_render::color::{HexColorError, HslRepresentation, SrgbColorSpace};
use serde::{Deserialize, Serialize};

/// Non-linear standard RGB with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(PartialEq, Serialize, Deserialize)]
pub struct Srgba {
    /// The red channel. [0.0, 1.0]
    pub red: f32,
    /// The green channel. [0.0, 1.0]
    pub green: f32,
    /// The blue channel. [0.0, 1.0]
    pub blue: f32,
    /// The alpha channel. [0.0, 1.0]
    pub alpha: f32,
}

impl StandardColor for Srgba {}

impl Srgba {
    // The standard VGA colors, with alpha set to 1.0.
    // https://en.wikipedia.org/wiki/Web_colors#Basic_colors

    /// <div style="background-color:rgb(0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const BLACK: Srgba = Srgba::new(0.0, 0.0, 0.0, 1.0);
    /// <div style="background-color:rgba(0%, 0%, 0%, 0%); width: 10px; padding: 10px; border: 1px solid;"></div>
    #[doc(alias = "transparent")]
    pub const NONE: Srgba = Srgba::new(0.0, 0.0, 0.0, 0.0);
    /// <div style="background-color:rgb(100%, 100%, 100%); width: 10px; padding: 10px; border: 1px solid;"></div>
    pub const WHITE: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);

    /// Construct a new [`Srgba`] color from components.
    ///
    /// # Arguments
    ///
    /// * `red` - Red channel. [0.0, 1.0]
    /// * `green` - Green channel. [0.0, 1.0]
    /// * `blue` - Blue channel. [0.0, 1.0]
    /// * `alpha` - Alpha channel. [0.0, 1.0]
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Construct a new [`Srgba`] color from (r, g, b) components, with the default alpha (1.0).
    ///
    /// # Arguments
    ///
    /// * `red` - Red channel. [0.0, 1.0]
    /// * `green` - Green channel. [0.0, 1.0]
    /// * `blue` - Blue channel. [0.0, 1.0]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    /// Return a copy of this color with the red channel set to the given value.
    pub const fn with_red(self, red: f32) -> Self {
        Self { red, ..self }
    }

    /// Return a copy of this color with the green channel set to the given value.
    pub const fn with_green(self, green: f32) -> Self {
        Self { green, ..self }
    }

    /// Return a copy of this color with the blue channel set to the given value.
    pub const fn with_blue(self, blue: f32) -> Self {
        Self { blue, ..self }
    }

    /// New `Srgba` from a CSS-style hexadecimal string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_color::Srgba;
    /// let color = Srgba::hex("FF00FF").unwrap(); // fuchsia
    /// let color = Srgba::hex("FF00FF7F").unwrap(); // partially transparent fuchsia
    ///
    /// // A standard hex color notation is also available
    /// assert_eq!(Srgba::hex("#FFFFFF").unwrap(), Srgba::new(1.0, 1.0, 1.0, 1.0));
    /// ```
    pub fn hex<T: AsRef<str>>(hex: T) -> Result<Self, HexColorError> {
        let hex = hex.as_ref();
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        match hex.len() {
            // RGB
            3 => {
                let [l, b] = u16::from_str_radix(hex, 16)?.to_be_bytes();
                let (r, g, b) = (l & 0x0F, (b & 0xF0) >> 4, b & 0x0F);
                Ok(Self::rgb_u8(r << 4 | r, g << 4 | g, b << 4 | b))
            }
            // RGBA
            4 => {
                let [l, b] = u16::from_str_radix(hex, 16)?.to_be_bytes();
                let (r, g, b, a) = ((l & 0xF0) >> 4, l & 0xF, (b & 0xF0) >> 4, b & 0x0F);
                Ok(Self::rgba_u8(
                    r << 4 | r,
                    g << 4 | g,
                    b << 4 | b,
                    a << 4 | a,
                ))
            }
            // RRGGBB
            6 => {
                let [_, r, g, b] = u32::from_str_radix(hex, 16)?.to_be_bytes();
                Ok(Self::rgb_u8(r, g, b))
            }
            // RRGGBBAA
            8 => {
                let [r, g, b, a] = u32::from_str_radix(hex, 16)?.to_be_bytes();
                Ok(Self::rgba_u8(r, g, b, a))
            }
            _ => Err(HexColorError::Length),
        }
    }

    /// Convert this color to CSS-style hexadecimal notation.
    pub fn to_hex(&self) -> String {
        let r = (self.red * 255.0).round() as u8;
        let g = (self.green * 255.0).round() as u8;
        let b = (self.blue * 255.0).round() as u8;
        let a = (self.alpha * 255.0).round() as u8;
        match a {
            255 => format!("#{:02X}{:02X}{:02X}", r, g, b),
            _ => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a),
        }
    }

    /// New `Srgba` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0, 255]
    /// * `g` - Green channel. [0, 255]
    /// * `b` - Blue channel. [0, 255]
    ///
    /// See also [`Srgba::new`], [`Srgba::rgba_u8`], [`Srgba::hex`].
    ///
    pub fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba_u8(r, g, b, u8::MAX)
    }

    // Float operations in const fn are not stable yet
    // see https://github.com/rust-lang/rust/issues/57241
    /// New `Srgba` from sRGB colorspace.
    ///
    /// # Arguments
    ///
    /// * `r` - Red channel. [0, 255]
    /// * `g` - Green channel. [0, 255]
    /// * `b` - Blue channel. [0, 255]
    /// * `a` - Alpha channel. [0, 255]
    ///
    /// See also [`Srgba::new`], [`Srgba::rgb_u8`], [`Srgba::hex`].
    ///
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / u8::MAX as f32,
            g as f32 / u8::MAX as f32,
            b as f32 / u8::MAX as f32,
            a as f32 / u8::MAX as f32,
        )
    }
}

impl Default for Srgba {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Luminance for Srgba {
    #[inline]
    fn luminance(&self) -> f32 {
        let linear: LinearRgba = (*self).into();
        linear.luminance()
    }

    #[inline]
    fn with_luminance(&self, luminance: f32) -> Self {
        let linear: LinearRgba = (*self).into();
        linear
            .with_luminance(luminance.nonlinear_to_linear_srgb())
            .into()
    }

    #[inline]
    fn darker(&self, amount: f32) -> Self {
        let linear: LinearRgba = (*self).into();
        linear.darker(amount).into()
    }

    #[inline]
    fn lighter(&self, amount: f32) -> Self {
        let linear: LinearRgba = (*self).into();
        linear.lighter(amount).into()
    }
}

impl Mix for Srgba {
    #[inline]
    fn mix(&self, other: &Self, factor: f32) -> Self {
        let n_factor = 1.0 - factor;
        Self {
            red: self.red * n_factor + other.red * factor,
            green: self.green * n_factor + other.green * factor,
            blue: self.blue * n_factor + other.blue * factor,
            alpha: self.alpha * n_factor + other.alpha * factor,
        }
    }
}

impl Alpha for Srgba {
    #[inline]
    fn with_alpha(&self, alpha: f32) -> Self {
        Self { alpha, ..*self }
    }

    #[inline]
    fn alpha(&self) -> f32 {
        self.alpha
    }
}

impl EuclideanDistance for Srgba {
    #[inline]
    fn distance_squared(&self, other: &Self) -> f32 {
        let dr = self.red - other.red;
        let dg = self.green - other.green;
        let db = self.blue - other.blue;
        dr * dr + dg * dg + db * db
    }
}

impl From<LinearRgba> for Srgba {
    #[inline]
    fn from(value: LinearRgba) -> Self {
        Self {
            red: value.red.linear_to_nonlinear_srgb(),
            green: value.green.linear_to_nonlinear_srgb(),
            blue: value.blue.linear_to_nonlinear_srgb(),
            alpha: value.alpha,
        }
    }
}

impl From<Hsla> for Srgba {
    fn from(value: Hsla) -> Self {
        let [r, g, b] =
            HslRepresentation::hsl_to_nonlinear_srgb(value.hue, value.saturation, value.lightness);
        Self::new(r, g, b, value.alpha)
    }
}

impl From<Oklaba> for Srgba {
    fn from(value: Oklaba) -> Self {
        Srgba::from(LinearRgba::from(value))
    }
}

impl From<Srgba> for bevy_render::color::LegacyColor {
    fn from(value: Srgba) -> Self {
        bevy_render::color::LegacyColor::Rgba {
            red: value.red,
            green: value.green,
            blue: value.blue,
            alpha: value.alpha,
        }
    }
}

impl From<bevy_render::color::LegacyColor> for Srgba {
    fn from(value: bevy_render::color::LegacyColor) -> Self {
        match value.as_rgba() {
            bevy_render::color::LegacyColor::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Srgba::new(red, green, blue, alpha),
            _ => unreachable!(),
        }
    }
}

impl From<Srgba> for [f32; 4] {
    fn from(color: Srgba) -> Self {
        [color.red, color.green, color.blue, color.alpha]
    }
}

impl From<Srgba> for Vec4 {
    fn from(color: Srgba) -> Self {
        Vec4::new(color.red, color.green, color.blue, color.alpha)
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::assert_approx_eq;

    use super::*;

    #[test]
    fn test_to_from_linear() {
        let srgba = Srgba::new(0.0, 0.5, 1.0, 1.0);
        let linear_rgba: LinearRgba = srgba.into();
        assert_eq!(linear_rgba.red, 0.0);
        assert_approx_eq!(linear_rgba.green, 0.2140, 0.0001);
        assert_approx_eq!(linear_rgba.blue, 1.0, 0.0001);
        assert_eq!(linear_rgba.alpha, 1.0);
        let srgba2: Srgba = linear_rgba.into();
        assert_eq!(srgba2.red, 0.0);
        assert_approx_eq!(srgba2.green, 0.5, 0.0001);
        assert_approx_eq!(srgba2.blue, 1.0, 0.0001);
        assert_eq!(srgba2.alpha, 1.0);
    }

    #[test]
    fn euclidean_distance() {
        // White to black
        let a = Srgba::new(0.0, 0.0, 0.0, 1.0);
        let b = Srgba::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(a.distance_squared(&b), 3.0);

        // Alpha shouldn't matter
        let a = Srgba::new(0.0, 0.0, 0.0, 1.0);
        let b = Srgba::new(1.0, 1.0, 1.0, 0.0);
        assert_eq!(a.distance_squared(&b), 3.0);

        // Red to green
        let a = Srgba::new(0.0, 0.0, 0.0, 1.0);
        let b = Srgba::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(a.distance_squared(&b), 1.0);
    }

    #[test]
    fn darker_lighter() {
        // Darker and lighter should be commutative.
        let color = Srgba::new(0.4, 0.5, 0.6, 1.0);
        let darker1 = color.darker(0.1);
        let darker2 = darker1.darker(0.1);
        let twice_as_dark = color.darker(0.2);
        assert!(darker2.distance_squared(&twice_as_dark) < 0.0001);

        let lighter1 = color.lighter(0.1);
        let lighter2 = lighter1.lighter(0.1);
        let twice_as_light = color.lighter(0.2);
        assert!(lighter2.distance_squared(&twice_as_light) < 0.0001);
    }

    #[test]
    fn hex_color() {
        assert_eq!(Srgba::hex("FFF"), Ok(Srgba::WHITE));
        assert_eq!(Srgba::hex("FFFF"), Ok(Srgba::WHITE));
        assert_eq!(Srgba::hex("FFFFFF"), Ok(Srgba::WHITE));
        assert_eq!(Srgba::hex("FFFFFFFF"), Ok(Srgba::WHITE));
        assert_eq!(Srgba::hex("000"), Ok(Srgba::BLACK));
        assert_eq!(Srgba::hex("000F"), Ok(Srgba::BLACK));
        assert_eq!(Srgba::hex("000000"), Ok(Srgba::BLACK));
        assert_eq!(Srgba::hex("000000FF"), Ok(Srgba::BLACK));
        assert_eq!(Srgba::hex("03a9f4"), Ok(Srgba::rgb_u8(3, 169, 244)));
        assert_eq!(Srgba::hex("yy"), Err(HexColorError::Length));
        assert_eq!(Srgba::hex("#f2a"), Ok(Srgba::rgb_u8(255, 34, 170)));
        assert_eq!(Srgba::hex("#e23030"), Ok(Srgba::rgb_u8(226, 48, 48)));
        assert_eq!(Srgba::hex("#ff"), Err(HexColorError::Length));
        assert_eq!(Srgba::hex("11223344"), Ok(Srgba::rgba_u8(17, 34, 51, 68)));
        assert_eq!(Srgba::hex("1234"), Ok(Srgba::rgba_u8(17, 34, 51, 68)));
        assert_eq!(Srgba::hex("12345678"), Ok(Srgba::rgba_u8(18, 52, 86, 120)));
        assert_eq!(Srgba::hex("4321"), Ok(Srgba::rgba_u8(68, 51, 34, 17)));

        assert!(matches!(Srgba::hex("yyy"), Err(HexColorError::Parse(_))));
        assert!(matches!(Srgba::hex("##fff"), Err(HexColorError::Parse(_))));
    }
}
