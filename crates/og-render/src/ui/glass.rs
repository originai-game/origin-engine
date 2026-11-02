//! Flat glass's presets (`docs/ui.md` §5.3): the three weights, their
//! elevations, the scrim and the two fallbacks as [`UiMode::Backdrop`]s and
//! tints, from the design language's material values (seeded from the
//! platform's `docs/design-language.md` §Materials and its tokens; a style
//! may use any other values through [`UiGlass`] itself).
//!
//! The material is clear and takes its colour from what passes under it:
//! a light tint over a modest blur with 200 % saturation (120 % dark), a 0.5 px
//! hairline just inside the edge, a soft shadow outside. Dark glass also
//! limits how bright the blurred scene may be (a sun behind it lifts it
//! evenly instead of staining it) and catches a light along its top edge,
//! the clean sci-fi direction's lit edge. No specular rim, sheen or
//! refraction. It mixes as CSS does ([`UiGlass::srgb_mix`]), so a
//! tint reads the same here as on the platform's pages.
//!
//! The engine's values diverge from the platform's in one place: the dark
//! `regular` tint is 64 % (the platform's 52 %), the least that holds the
//! scheme's `fg` at 4.5:1 on a bar over a bright sky. `glass-thin` keeps
//! the platform's clear tints and carries icons and play discs only, never
//! text: holding text there would take a tint as dense as `regular`.

use super::{UiGlass, UiGlassShadow, UiMode};
use glam::Vec2;

/// How heavy a glass surface is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiGlassWeight {
    /// 10 px of blur and the lightest tint: icons and play discs over
    /// art, never text.
    Thin,
    /// 22 px: bars, tab bars, capsules.
    #[default]
    Regular,
    /// 32 px and a denser tint: what carries text (menus, popovers,
    /// dialogs, sheets, toasts).
    Thick,
}

/// The soft shadow a glass surface casts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiElevation {
    /// The hairline alone (in the light scheme an outer one).
    #[default]
    Raised,
    /// One soft, short shadow: capsules and menus.
    Float,
    /// Two, deeper: dialogs and sheets.
    Lift,
}

/// What the player's settings ask of glass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiGlassFallback {
    #[default]
    None,
    /// Nothing see-through: the solid raised surface with the hairline and
    /// shadow; the scrim does not blur.
    ReducedTransparency,
    /// Solid, with a 1 px strong border.
    IncreasedContrast,
}

/// One glass surface of the design language.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiGlassStyle {
    pub weight: UiGlassWeight,
    pub elevation: UiElevation,
    /// The dark scheme (glass over art is dark in both).
    pub dark: bool,
    pub fallback: UiGlassFallback,
}

/// sRGB bytes and a straight alpha as linear RGBA.
fn srgb(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
    let l = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    [l(r), l(g), l(b), a]
}

/// A CSS `box-shadow` of black at `alpha`.
fn shadow(y: f32, blur: f32, spread: f32, alpha: f32) -> UiGlassShadow {
    UiGlassShadow {
        offset: Vec2::new(0.0, y),
        blur,
        spread,
        color: [0.0, 0.0, 0.0, alpha],
    }
}

impl UiGlassStyle {
    /// The blur, CSS px, as the design language names it (CSS's
    /// `blur()`, a Gaussian's deviation).
    pub fn css_blur(self) -> f32 {
        match self.weight {
            UiGlassWeight::Thin => 10.0,
            UiGlassWeight::Regular => 22.0,
            UiGlassWeight::Thick => 32.0,
        }
    }

    /// The backdrop and its tint (the shape's solid paint, linear RGBA).
    pub fn backdrop(self) -> (UiMode, [f32; 4]) {
        let dark = self.dark;
        let tint = match (self.weight, dark) {
            (UiGlassWeight::Thin, false) => srgb(255, 255, 255, 0.36),
            (UiGlassWeight::Regular, false) => srgb(250, 250, 252, 0.58),
            (UiGlassWeight::Thick, false) => srgb(246, 246, 248, 0.8),
            (UiGlassWeight::Thin, true) => srgb(44, 44, 48, 0.28),
            (UiGlassWeight::Regular, true) => srgb(30, 30, 34, 0.64),
            (UiGlassWeight::Thick, true) => srgb(28, 28, 30, 0.78),
        };
        let edge_color = if dark {
            [1.0, 1.0, 1.0, 0.08]
        } else {
            [0.0, 0.0, 0.0, 0.07]
        };
        let none = UiGlassShadow::default();
        let shadows = match (self.elevation, dark) {
            (UiElevation::Raised, true) => [none, none],
            (UiElevation::Raised, false) => [shadow(0.0, 0.0, 0.5, 0.08), none],
            (UiElevation::Float, true) => [shadow(8.0, 24.0, -8.0, 0.6), none],
            (UiElevation::Float, false) => {
                [shadow(0.0, 0.0, 0.5, 0.06), shadow(6.0, 20.0, -8.0, 0.16)]
            }
            (UiElevation::Lift, true) => [
                shadow(10.0, 24.0, -8.0, 0.6),
                shadow(32.0, 64.0, -16.0, 0.8),
            ],
            (UiElevation::Lift, false) => [
                shadow(10.0, 24.0, -8.0, 0.25),
                shadow(32.0, 64.0, -16.0, 0.4),
            ],
        };
        let raised = if dark {
            srgb(0x2c, 0x2c, 0x2e, 1.0)
        } else {
            [1.0; 4]
        };
        // Dark glass: a bright scene behind it lifts it evenly rather than
        // staining it (the sun through a menu read as a brown blob at
        // 200 % saturation), and the top edge catches a light.
        let (backdrop_max, edge_top) = if dark { (0.30, 0.22) } else { (0.0, 0.0) };
        let mut glass = UiGlass {
            edge: 0.5,
            edge_color,
            shadows,
            solid: false,
            srgb_mix: true,
            backdrop_max,
            edge_top,
        };
        let (blur, saturation, tint) = match self.fallback {
            // Dark glass saturates 120 %, not CSS's 200 %: over a game's
            // bright sky the doubled colour came through a dark tint as a
            // brown stain.
            UiGlassFallback::None => (
                GLASS_BLUR_SCALE * self.css_blur(),
                if dark { 1.2 } else { 2.0 },
                tint,
            ),
            UiGlassFallback::ReducedTransparency => {
                glass.solid = true;
                (0.0, 1.0, raised)
            }
            UiGlassFallback::IncreasedContrast => {
                glass.solid = true;
                glass.edge = 1.0;
                glass.edge_color = if dark {
                    srgb(0x54, 0x54, 0x58, 1.0)
                } else {
                    srgb(0xc4, 0xc4, 0xc9, 1.0)
                };
                (0.0, 1.0, raised)
            }
        };
        (
            UiMode::Backdrop {
                blur,
                saturation,
                brightness: 1.0,
                glass,
            },
            tint,
        )
    }
}

/// The engine's blur radius ([`UiMode::Backdrop`]'s `blur`, the pyramid's
/// reach) per unit of CSS `blur()` (a Gaussian's deviation): a pyramid
/// level of reach 2σ shows about the softness of CSS's blur of σ.
pub const GLASS_BLUR_SCALE: f32 = 2.0;

/// The scrim under a dialog: it dims and softly blurs what it covers
/// (8 px, 120 % saturation); with reduced transparency (or increased
/// contrast) it dims without blurring, a plain fill. The mode and its
/// colour (linear RGBA).
pub fn ui_scrim(dark: bool, fallback: UiGlassFallback) -> (UiMode, [f32; 4]) {
    let tint = [0.0, 0.0, 0.0, if dark { 0.42 } else { 0.22 }];
    if fallback != UiGlassFallback::None {
        return (UiMode::Fill, tint);
    }
    (
        UiMode::Backdrop {
            blur: GLASS_BLUR_SCALE * 8.0,
            saturation: 1.2,
            brightness: 1.0,
            glass: UiGlass {
                srgb_mix: true,
                ..UiGlass::default()
            },
        },
        tint,
    )
}
