//! The theme: the platform's design tokens (generated into [`crate::theme`])
//! with the few conversions the renderer needs. No value is defined here.

use crate::theme as tokens;
pub use crate::theme::{
    Density, Durations, Hues, Palette, Radius, Space, Spring, Srgba, TypeRamp, TypeStep,
};

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

    /// The same colour with its alpha multiplied by `k`.
    pub fn fade(self, k: f32) -> Srgba {
        Srgba {
            rgb: self.rgb,
            alpha: self.alpha * k,
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

    pub const WHITE: Srgba = Srgba::rgb(255, 255, 255);
    pub const TRANSPARENT: Srgba = Srgba {
        rgb: [0, 0, 0],
        alpha: 0.0,
    };
}

/// Light or dark: which palette the theme uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scheme {
    Light,
    /// Games run on a stage; dark is the default.
    #[default]
    Dark,
}

/// Which control sizes apply (`docs/design-language.md` §Density and type).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DensityMode {
    /// Touch on a coarse pointer or a viewport under 768 CSS pixels, compact otherwise.
    #[default]
    Auto,
    Compact,
    Comfortable,
    Touch,
}

/// Everything a widget reads its look from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub scheme: Scheme,
    pub palette: Palette,
    pub hues: Hues,
    pub space: Space,
    pub radius: Radius,
    pub type_ramp: TypeRamp,
    pub density: DensityMode,
    pub durations_ms: Durations,
    /// How far a pressed control shrinks (`vars.press-scale`).
    pub press_scale: f32,
}

impl Default for Theme {
    fn default() -> Theme {
        Theme::dark()
    }
}

impl Theme {
    pub fn light() -> Theme {
        Theme::new(Scheme::Light)
    }

    pub fn dark() -> Theme {
        Theme::new(Scheme::Dark)
    }

    pub fn new(scheme: Scheme) -> Theme {
        let (palette, hues) = match scheme {
            Scheme::Light => (tokens::LIGHT, tokens::HUES_LIGHT),
            Scheme::Dark => (tokens::DARK, tokens::HUES_DARK),
        };
        Theme {
            scheme,
            palette,
            hues,
            space: tokens::SPACE,
            radius: tokens::RADIUS,
            type_ramp: tokens::TYPE_RAMP,
            density: DensityMode::Auto,
            durations_ms: tokens::DURATIONS_MS,
            press_scale: tokens::PRESS_SCALE,
        }
    }

    /// The control sizes for a viewport.
    pub fn sizes(&self, viewport: &crate::Viewport) -> Density {
        match self.density {
            DensityMode::Compact => tokens::COMPACT,
            DensityMode::Comfortable => tokens::COMFORTABLE,
            DensityMode::Touch => tokens::TOUCH,
            DensityMode::Auto => {
                // "coarse pointers and widths under 768px raise controls to touch size"
                if viewport.coarse_pointer || viewport.width < 768.0 {
                    tokens::TOUCH
                } else {
                    tokens::COMPACT
                }
            }
        }
    }
}
