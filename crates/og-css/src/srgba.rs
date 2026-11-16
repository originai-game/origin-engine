//! An sRGB colour as a style authors it, and the conversions the UI and
//! the renderer need.

use color::{AlphaColor, DisplayP3, DynamicColor, LinearSrgb, Srgb};
use og_core::color::{linear_to_srgb8, srgb8_to_linear};

/// An sRGB colour as a style authors it: 8-bit channels, straight alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgba {
    pub rgb: [u8; 3],
    pub alpha: f32,
    /// The colour in Display P3 (8-bit, on the sRGB curve) where it lies
    /// outside sRGB and the output presents P3 ([`crate::color::Gamut`]):
    /// `rgb` is then its gamut mapping into sRGB, which contrast, blending
    /// and mixing read, and [`Self::linear`] draws this one. `None` for a
    /// colour inside sRGB, or on an sRGB output.
    pub p3: Option<[u8; 3]>,
}

impl Srgba {
    /// Linear RGBA (straight alpha), what og-render's sprites and text take:
    /// a P3 colour as extended linear sRGB (a channel may be negative).
    pub fn linear(self) -> [f32; 4] {
        let [r, g, b] = match self.p3 {
            Some(p3) => {
                let p3 = p3.map(|v| f32::from(v) / 255.0);
                AlphaColor::<DisplayP3>::new([p3[0], p3[1], p3[2], 1.0])
                    .convert::<LinearSrgb>()
                    .components
                    .first_chunk::<3>()
                    .copied()
                    .expect("four components")
            }
            None => self.rgb.map(srgb8_to_linear),
        };
        [r, g, b, self.alpha]
    }

    /// The same colour with its alpha multiplied by `k` (at most opaque).
    pub fn fade(self, k: f32) -> Srgba {
        Srgba {
            alpha: (self.alpha * k).min(1.0),
            ..self
        }
    }

    /// An opaque colour from 8-bit sRGB channels (for game content; theme
    /// colours come from the palette).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Srgba {
        Srgba {
            rgb: [r, g, b],
            alpha: 1.0,
            p3: None,
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

    /// A colour from linear RGBA (straight alpha), rounded to 8-bit sRGB;
    /// extended linear sRGB outside sRGB (a negative channel, as
    /// [`Self::linear`] gives a P3 colour) keeps its P3, clipped to P3.
    pub fn from_linear(c: [f32; 4]) -> Srgba {
        let rgb = [c[0], c[1], c[2]];
        let p3 = rgb
            .iter()
            .any(|v| !(-1e-4..=1.0 + 1e-4).contains(v))
            .then(|| {
                let p3 = AlphaColor::<LinearSrgb>::new([c[0], c[1], c[2], 1.0])
                    .convert::<DisplayP3>()
                    .components;
                [p3[0], p3[1], p3[2]].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
            });
        Srgba {
            rgb: rgb.map(linear_to_srgb8),
            alpha: c[3].clamp(0.0, 1.0),
            p3,
        }
    }

    /// This colour composited over an opaque `below` in linear light, as
    /// og-render blends it, opaque.
    pub fn over_linear(self, below: Srgba) -> Srgba {
        let (t, b) = (self.linear(), below.linear());
        let a = self.alpha.clamp(0.0, 1.0);
        let e = |k: usize| linear_to_srgb8(t[k] * a + b[k] * (1.0 - a));
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
            p3: None,
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

    /// CSS `contrast-color()` of this colour as an opaque ground (its alpha
    /// is not read): white or black, whichever has the greater WCAG 2 ratio
    /// on it, white on a tie. The two ratios multiply to 21, so the pick
    /// reads at least √21 ≈ 4.58:1 on any ground.
    pub fn contrast_color(self) -> Srgba {
        let ground = Srgba { alpha: 1.0, ..self };
        if Srgba::WHITE.contrast(ground) >= Srgba::BLACK.contrast(ground) {
            Srgba::WHITE
        } else {
            Srgba::BLACK
        }
    }

    /// The APCA lightness contrast (Lc) of this colour as text on the
    /// opaque `ground`: positive for dark text on a light ground, negative
    /// for light on dark, 0 under |Lc 10|; about 106 for black on white and
    /// −108 for white on black. The base formula 0.0.98G-4g with its
    /// constants as apca-w3 0.1.9 states them (Somers, Myndex); written
    /// here from that statement, not copied.
    pub fn apca(self, ground: Srgba) -> f32 {
        // sRGB to APCA's screen luminance: a plain 2.4 power, then a soft
        // clamp of near black.
        let y = |c: Srgba| {
            let [r, g, b] = c.rgb.map(|v| libm::powf(f32::from(v) / 255.0, 2.4));
            let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
            if y > 0.022 {
                y
            } else {
                y + libm::powf(0.022 - y, 1.414)
            }
        };
        let (text, ground) = (y(self), y(ground));
        if (ground - text).abs() < 0.0005 {
            return 0.0;
        }
        let lc = if ground > text {
            let s = (libm::powf(ground, 0.56) - libm::powf(text, 0.57)) * 1.14;
            if s < 0.1 { 0.0 } else { s - 0.027 }
        } else {
            let s = (libm::powf(ground, 0.65) - libm::powf(text, 0.62)) * 1.14;
            if s > -0.1 { 0.0 } else { s + 0.027 }
        };
        lc * 100.0
    }

    pub const WHITE: Srgba = Srgba::rgb(255, 255, 255);
    pub const BLACK: Srgba = Srgba::rgb(0, 0, 0);
    pub const TRANSPARENT: Srgba = Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
        p3: None,
    };
}

impl From<Srgba> for AlphaColor<Srgb> {
    fn from(c: Srgba) -> AlphaColor<Srgb> {
        let [r, g, b] = c.rgb.map(|v| f32::from(v) / 255.0);
        AlphaColor::new([r, g, b, c.alpha])
    }
}

impl From<Srgba> for DynamicColor {
    /// In Display P3 where the colour keeps it, so a mix of it keeps its range.
    fn from(c: Srgba) -> DynamicColor {
        match c.p3 {
            Some(p3) => {
                let [r, g, b] = p3.map(|v| f32::from(v) / 255.0);
                DynamicColor::from_alpha_color(AlphaColor::<DisplayP3>::new([r, g, b, c.alpha]))
            }
            None => DynamicColor::from_alpha_color(AlphaColor::<Srgb>::from(c)),
        }
    }
}

impl From<AlphaColor<Srgb>> for Srgba {
    /// Rounded to 8-bit channels (clipped), the alpha clamped.
    fn from(c: AlphaColor<Srgb>) -> Srgba {
        let [r, g, b, alpha] = c.components;
        Srgba {
            rgb: [r, g, b].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
            alpha: alpha.clamp(0.0, 1.0),
            p3: None,
        }
    }
}

/// A CSS hex colour: `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`.
pub fn parse_color(s: &str) -> Option<Srgba> {
    s.starts_with('#')
        .then(|| color::parse_color(s).ok())
        .flatten()
        .map(|c| c.to_alpha_color::<Srgb>().into())
}
