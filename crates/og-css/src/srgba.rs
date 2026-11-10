//! An sRGB colour as a style authors it, and the conversions the UI and
//! the renderer need.

/// An sRGB colour as a style authors it: 8-bit channels, straight alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgba {
    pub rgb: [u8; 3],
    pub alpha: f32,
}

impl Srgba {
    /// Linear RGBA (straight alpha), what og-render's sprites and text take.
    pub fn linear(self) -> [f32; 4] {
        let c = |v: u8| {
            let s = f32::from(v) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        [c(self.rgb[0]), c(self.rgb[1]), c(self.rgb[2]), self.alpha]
    }

    /// The same colour with its alpha multiplied by `k` (at most opaque).
    pub fn fade(self, k: f32) -> Srgba {
        Srgba {
            rgb: self.rgb,
            alpha: (self.alpha * k).min(1.0),
        }
    }

    /// An opaque colour from 8-bit sRGB channels (for game content; theme
    /// colours come from the palette).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Srgba {
        Srgba {
            rgb: [r, g, b],
            alpha: 1.0,
        }
    }

    /// This colour composited over an opaque `below` (straight alpha), opaque.
    pub fn over(self, below: Srgba) -> Srgba {
        let a = self.alpha.clamp(0.0, 1.0);
        let mix = |t: u8, b: u8| (f32::from(t) * a + f32::from(b) * (1.0 - a)).round() as u8;
        Srgba::rgb(
            mix(self.rgb[0], below.rgb[0]),
            mix(self.rgb[1], below.rgb[1]),
            mix(self.rgb[2], below.rgb[2]),
        )
    }

    /// A colour from linear RGBA (straight alpha), rounded to 8-bit sRGB.
    pub fn from_linear(c: [f32; 4]) -> Srgba {
        let e = |v: f32| {
            let s = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (s.clamp(0.0, 1.0) * 255.0).round() as u8
        };
        Srgba {
            rgb: [e(c[0]), e(c[1]), e(c[2])],
            alpha: c[3].clamp(0.0, 1.0),
        }
    }

    /// This colour composited over an opaque `below` in linear light, as
    /// og-render blends it, opaque.
    pub fn over_linear(self, below: Srgba) -> Srgba {
        let (t, b) = (self.linear(), below.linear());
        let a = self.alpha.clamp(0.0, 1.0);
        let e = |k: usize| {
            let v = t[k] * a + b[k] * (1.0 - a);
            let s = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (s.clamp(0.0, 1.0) * 255.0).round() as u8
        };
        Srgba::rgb(e(0), e(1), e(2))
    }

    /// This colour at alpha `k` over `below` as a browser composites it (in
    /// sRGB, a CSS `rgb(… / 16%)` wash), kept translucent for og-render,
    /// which composites in linear light: the alpha whose linear blend over
    /// `below` has the browser's luminance, so the art under it still
    /// shows. A linear 16 % of a light ink over a dark ground lifts the
    /// ground to a sixth of the ink's luminance (no ink then reads 4.5:1 on
    /// it, white included, on the darker packs); the browser's 16 % lifts
    /// it a few percent. Over a light ground it darkens more than linear.
    pub fn tint_over(self, below: Srgba, k: f32) -> Srgba {
        let (lc, lb) = (self.luminance(), below.luminance());
        let mixed = self.fade(k).over(below).luminance();
        if (lc - lb).abs() < 1e-4 {
            return self.fade(k);
        }
        self.fade(((mixed - lb) / (lc - lb)).clamp(0.0, 1.0) / self.alpha.max(1e-4))
    }

    /// This translucent fill, stated for og-render's linear-light blending
    /// as a theme's are (`packages/design`'s `linear` values), `k` times as
    /// strong as it reads: the CSS alpha it draws like over mid grey (the
    /// tokens' ground for a scrim), times `k`, restated for linear light.
    /// [`Srgba::fade`] scales the linear alpha itself, which overshoots: a
    /// dark scrim's 0.69 (CSS 0.42) times 1.6 is opaque, not CSS 0.67.
    pub fn deepen(self, k: f32) -> Srgba {
        let grey = Srgba::rgb(128, 128, 128);
        let (lc, lb) = (Srgba { alpha: 1.0, ..self }.luminance(), grey.luminance());
        if (lc - lb).abs() < 1e-4 {
            return self.fade(k);
        }
        let css = |s: f32| Srgba { alpha: s, ..self }.over(grey).luminance();
        // The CSS alpha drawing the luminance this one draws in linear light.
        let drawn = lb + (lc - lb) * self.alpha.clamp(0.0, 1.0);
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if (css(mid) - lb).abs() < (drawn - lb).abs() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let want = css(((lo + hi) / 2.0 * k).min(1.0));
        Srgba {
            alpha: ((want - lb) / (lc - lb)).clamp(0.0, 1.0),
            ..self
        }
    }

    /// Mixed towards `to` by `t` (0–1), keeping this colour's alpha.
    pub fn mix(self, to: Srgba, t: f32) -> Srgba {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
        Srgba {
            rgb: [
                m(self.rgb[0], to.rgb[0]),
                m(self.rgb[1], to.rgb[1]),
                m(self.rgb[2], to.rgb[2]),
            ],
            alpha: self.alpha,
        }
    }

    /// This ink, or the least mix of it towards its own end of the scale
    /// (white for an ink lighter than `ground`, black for a darker one) that
    /// reads `bar`:1 on the opaque `ground`; `None` when even that end does
    /// not. Value before hue: the ink lightens or darkens, keeping its hue
    /// as far as the bar allows. Called with the two swapped, it moves the
    /// ground away from the ink instead.
    pub fn readable_on(self, ground: Srgba, bar: f32) -> Option<Srgba> {
        if self.contrast(ground) >= bar {
            return Some(self);
        }
        let end = if self.luminance() >= ground.luminance() {
            Srgba::WHITE
        } else {
            Srgba::BLACK
        };
        if end.contrast(ground) < bar {
            return None;
        }
        // The contrast grows with the mix: the least one at the bar.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..16 {
            let mid = (lo + hi) / 2.0;
            if self.mix(end, mid).contrast(ground) >= bar {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        // Rounded to bytes, a step may still sit just under: step on.
        let mut t = hi;
        let mut c = self.mix(end, t);
        while c.contrast(ground) < bar && t < 1.0 {
            t = (t + 1.0 / 255.0).min(1.0);
            c = self.mix(end, t);
        }
        Some(c)
    }

    /// The WCAG contrast ratio of this colour and `other` (1 to 21).
    pub fn contrast(self, other: Srgba) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// WCAG relative luminance (0 black – 1 white).
    pub fn luminance(self) -> f32 {
        let c = self.linear();
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    pub const WHITE: Srgba = Srgba::rgb(255, 255, 255);
    pub const BLACK: Srgba = Srgba::rgb(0, 0, 0);
    pub const TRANSPARENT: Srgba = Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
    };
}
