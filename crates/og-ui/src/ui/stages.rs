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

/// The frame's output: what the host takes after it (the frame, warnings, events, feedback, clipboard) and its counts.
#[derive(Clone, Debug)]
pub(crate) struct Out {
    pub(crate) frame: UiFrame,
    pub(crate) warnings: Vec<String>,
    /// The (face, character) pairs [`UiFrame::missing_glyphs`] named.
    pub(crate) glyphs_missed: std::collections::BTreeSet<(og_draw::FontHandle, char)>,
    /// This frame's motion events (into `UiFrame::events`).
    pub(crate) events: Vec<UiEvent>,
    /// UI events the style gives a sound or a haptic, not yet taken by the
    /// host ([`Ui::take_feedback`]), each with where it is in a rising
    /// cascade (0–1; NaN: a random pitch in the event's range).
    pub(crate) feedback: Vec<(crate::tokens::SoundEvent, f32, crate::tokens::Feedback)>,
    /// The haptic patterns played since the host last took them
    /// ([`Ui::take_haptics`]).
    pub(crate) haptic_out: Vec<crate::tokens::Haptic>,
    /// Text a copy or cut took, for the host's clipboard ([`Ui::take_clipboard`]).
    pub(crate) clipboard: Option<String>,
    /// This frame's counts (into `UiFrame::stats`).
    pub(crate) stats: LayoutStats,
}

/// Documents: their mounts, the layer stack and the reactive session.
#[derive(Clone, Debug)]
pub(crate) struct Docs {
    /// Documents, their mounts and the layer stack (docs/ui.md §2.1, §2.1).
    pub(crate) host: crate::doc::build::DocHost,
    pub(crate) reactive: crate::reactive::session::Session,
}

/// Style: the part rules and tokens, the motion pass's memory, timelines and the plays asked for.
#[derive(Clone, Debug)]
pub(crate) struct Style {
    /// Global CSS part rules in cascade order and their at-rule descriptors.
    pub(crate) part_rules: Vec<crate::doc::StyleRule>,
    pub(crate) part_sheet: crate::doc::at_rules::Sheet,
    pub(crate) root_tokens: Vec<(String, String)>,
    pub(crate) root_type_tokens: [Vec<(String, String)>; 4],
    /// Part samples read one before-change state throughout a frame. Commit
    /// their next state together after rendering the owning nodes.
    pub(crate) part_commits: std::cell::RefCell<
        std::collections::BTreeMap<
            (u64, &'static str),
            std::sync::Arc<crate::doc::build::StyleCommit>,
        >,
    >,
    pub(crate) css_sounds: std::collections::BTreeMap<String, u64>,
    /// Timelines by name hash (`define_timeline`): configuration, like the theme.
    pub(crate) timelines: std::collections::BTreeMap<u64, (String, Timeline)>,
    /// Plays asked for this tick, flattened at `end`.
    pub(crate) pending_plays: Vec<crate::motion::run::PendingPlay>,
    /// The motion pass's memory between ticks.
    pub(crate) motion: crate::layout::MotionCache,
}

/// Paint: its reuse cache, the part and decoration draws, the map being drawn and the last frame's draws and inspection.
#[derive(Clone, Debug)]
pub(crate) struct Paint {
    /// This frame's command ranges kept from last frame's paint, in order.
    pub(crate) kept_spans: Vec<std::ops::Range<usize>>,
    /// What each part of this frame draws through the UI pass, by node
    /// (made at `end` from its dressing, or the tokens).
    pub(crate) part_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// Parts whose computed paint changed since the preceding frame.
    pub(crate) part_changed: std::collections::HashSet<usize>,
    /// This frame's draws of elements with CSS box decorations: their
    /// draw or dressing with the decorations over it.
    pub(crate) decor_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// The map of the element whose commands are being made: a nine-slice
    /// is cut at its size on the screen, its corners kept crisp.
    pub(crate) drawing: Affine,
    pub(crate) prev_draws: std::collections::HashMap<usize, crate::paint::ElementDraw>,
    /// The last frame's inspection, made when asked for (`inspect`).
    pub(crate) inspection: inspect::Inspection,
    /// Last frame's paint, for reuse.
    pub(crate) cache: crate::layout::PaintCache,
}
