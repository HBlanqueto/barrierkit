#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self::rgba8(0, 0, 0, 0.0);

    pub fn mix(self, other: Self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * amount,
            g: self.g + (other.g - self.g) * amount,
            b: self.b + (other.b - self.b) * amount,
            a: self.a,
        }
    }

    pub fn darken(self, amount: f32) -> Self {
        self.mix(Self::rgb8(0, 0, 0), amount)
    }

    pub fn lighten(self, amount: f32) -> Self {
        self.mix(Self::rgb8(255, 255, 255), amount)
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 1.0)
    }

    pub const fn rgba8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    pub fn to_rgba8(self) -> [u8; 4] {
        [
            (self.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.b * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn premultiplied(self) -> [f32; 4] {
        [self.r * self.a, self.g * self.a, self.b * self.a, self.a]
    }

    pub fn to_linear(self) -> Self {
        let decode = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        Self {
            r: decode(self.r),
            g: decode(self.g),
            b: decode(self.b),
            a: self.a,
        }
    }

    pub fn from_linear(linear: Self) -> Self {
        let encode = |channel: f32| {
            let channel = channel.clamp(0.0, 1.0);
            if channel <= 0.003_130_8 {
                12.92 * channel
            } else {
                1.055 * channel.powf(1.0 / 2.4) - 0.055
            }
        };
        Self {
            r: encode(linear.r),
            g: encode(linear.g),
            b: encode(linear.b),
            a: linear.a,
        }
    }

    pub fn relative_luminance(self) -> f32 {
        let linear = self.to_linear();
        0.2126 * linear.r + 0.7152 * linear.g + 0.0722 * linear.b
    }

    pub fn contrast_ratio(self, other: Self) -> f32 {
        let a = self.relative_luminance();
        let b = other.relative_luminance();
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let lightness = (max + min) / 2.0;
        let delta = max - min;
        if delta == 0.0 {
            return (0.0, 0.0, lightness);
        }
        let saturation = if lightness > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };
        let hue = if max == self.r {
            ((self.g - self.b) / delta + if self.g < self.b { 6.0 } else { 0.0 }) / 6.0
        } else if max == self.g {
            ((self.b - self.r) / delta + 2.0) / 6.0
        } else {
            ((self.r - self.g) / delta + 4.0) / 6.0
        };
        (hue, saturation, lightness)
    }

    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        }
        let (r, g, b) = if saturation == 0.0 {
            (lightness, lightness, lightness)
        } else {
            let q = if lightness < 0.5 {
                lightness * (1.0 + saturation)
            } else {
                lightness + saturation - lightness * saturation
            };
            let p = 2.0 * lightness - q;
            (
                hue_to_rgb(p, q, hue + 1.0 / 3.0),
                hue_to_rgb(p, q, hue),
                hue_to_rgb(p, q, hue - 1.0 / 3.0),
            )
        };
        Self { r, g, b, a: alpha }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn premultiplied_opaque_is_unchanged() {
        let c = super::Rgba::rgb8(128, 64, 255);
        let [r, g, b, a] = c.premultiplied();
        assert!((r - c.r).abs() < 0.004);
        assert!((g - c.g).abs() < 0.004);
        assert!((b - c.b).abs() < 0.004);
        assert!((a - c.a).abs() < 0.001);
    }

    #[test]
    fn premultiplied_honours_alpha() {
        let c = super::Rgba::rgba8(255, 128, 0, 0.5);
        let [r, g, b, a] = c.premultiplied();
        assert!((r - 0.5).abs() < 0.004, "red should be halved");
        assert!((g - 0.25).abs() < 0.004, "green should be halved");
        assert!(b < 0.004);
        assert!((a - 0.5).abs() < 0.001);
    }
}
