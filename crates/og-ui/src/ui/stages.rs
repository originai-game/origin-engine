//! The stage states `Ui` is made of (#41): each owns what one stage of
//! the frame writes.

use super::*;

/// Configuration: the theme, fonts, viewport and locale every stage reads.
#[derive(Clone, Debug)]
pub(crate) struct Env {
    /// The theme as configured ([`Ui::set_theme`]); `theme` is it with the
    /// viewport's preferences applied.
    pub(crate) configured: Theme,
    pub(crate) theme: Theme,
    pub(crate) fonts: Fonts,
    pub(crate) ticks_per_second: u32,
    pub(crate) viewport: Viewport,
    /// The og-ui unit of the frame being built, CSS px (`begin`).
    pub(crate) unit: f32,
    /// The writing direction: right to left mirrors the layout and turns
    /// text alignment round (og.i18n's locale, or og.ui.configure).
    pub(crate) direction: og_i18n::Direction,
    /// Active BCP-47 locale, for display text casing.
    pub(crate) locale: String,
    pub(crate) icons: IconAtlas,
    /// The style's icon names, which its colour tokens' icon cues index
    /// (`Ui::set_cue_icons`); empty: the atlas's own names.
    pub(crate) cue_icons: Vec<String>,
    /// Pictures rich text draws by name (`[img=…]`).
    pub(crate) images: std::collections::BTreeMap<String, InlineImage>,
    /// The configured style's name (empty: the default style).
    pub(crate) style_name: String,
    /// The player's haptic strength (0: off, 1: as the style sets it).
    pub(crate) haptics: f32,
    /// The widest the HUD band is, over its height ([`Ui::hud_area`]).
    pub(crate) hud_max_aspect: f32,
    /// The host opens a file picker for `<input type="file">`
    /// ([`Ui::set_file_picker`]).
    pub(crate) file_picker: bool,
    pub(crate) toast_anchor: Anchor,
}
