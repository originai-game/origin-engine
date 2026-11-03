//! The game widget set over the core (`ui.rs`): layers (anchors, dialogs,
//! menus, toasts), transitions, virtualised lists and grids, selection
//! controls, HUD meters, touch controls and the dialogue box. Every widget
//! is one call with a desc whose zero value is the default, returns what
//! happened this tick, and keeps what must persist between ticks in the
//! UI state (so snapshots replay it). Colours are the theme's tokens.

use crate::id::Id;
use crate::layout::{
    Align, Anchor, Content, Cross, Direction, Element, Float, FontRole, Frame, Insets, NONE, Shape,
    Size, TextAlign, Widget,
};
use crate::motion::{TransitionKind, ease_exit, ease_out, fx};
use crate::style::{READABLE, Srgba};
use crate::ui::{Anim, Response, Toast, Ui, mix_srgba};
use og_core::hash::StableHasher;
use og_render::TextureHandle;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

// ── descs and answers ───────────────────────────────────────────────────────

/// A spotlight: the surface dimmed except a hole around a target.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpotlightDesc {
    /// Room around the target; 0: `space.sm`.
    pub padding: f32,
}

/// An anchored group: a layer at an anchor of the safe area (HUD corners,
/// a centred title, the bottom bar of touch controls).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnchorDesc {
    pub anchor: Anchor,
    /// Inset from the safe area's edges; `None`: the density's gutter.
    pub margin: Option<f32>,
    /// Moves it after placing (CSS px, y down).
    pub offset: (f32, f32),
    pub direction: Direction,
    pub gap: f32,
    pub padding: Insets,
    pub align: Align,
    pub cross: Cross,
    pub background: Option<Srgba>,
    pub radius: f32,
    pub width: Size,
    pub height: Size,
    /// Draws over dialogs and spotlights (a tutorial callout, a dragged item).
    pub over: bool,
    /// Draws in the tooltip layer, over every other layer including the
    /// toast stack (a card the player opened).
    pub tooltip: bool,
    /// Steps clear of the other anchored groups and the host's control
    /// (a drift score over the car, which must not cover the map).
    pub avoid: bool,
    /// The frame its background takes (`None`: the panel's).
    pub frame: Option<crate::style_file::StylePart>,
    /// Shown settled the first time it is built: no style entrance or
    /// stagger (the screen a game opens on, which the web draws behind its
    /// play gate).
    pub instant: bool,
}

/// How a transition moves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransitionDesc {
    pub kind: TransitionKind,
    /// 0: `motion.duration.base`.
    pub duration_ms: f32,
    /// How far slides travel (0: `space.xl`).
    pub distance: f32,
    /// Shown at once the first time (no entrance).
    pub instant: bool,
}

/// Where a transition is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Presence {
    /// Build its content this tick (shown, entering or leaving).
    pub visible: bool,
    /// 0 gone … 1 settled.
    pub progress: f32,
    pub entering: bool,
    pub leaving: bool,
}

/// A dialog's look and rules.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModalDesc {
    /// A title over the content (empty: none).
    pub title: String,
    /// The panel's width; `None`: 420 CSS px, at most the surface less its gutters.
    pub width: Option<Size>,
    /// Back (Escape, gamepad B) and the backdrop do not dismiss it.
    pub locked: bool,
    /// A press on the backdrop does not dismiss it.
    pub ignore_backdrop: bool,
    /// How it enters and leaves (the zero value: scale).
    pub transition: Option<TransitionKind>,
    /// The backdrop's fill; `None`: `scrim`.
    pub backdrop: Option<Srgba>,
    /// No panel: the content sits on the backdrop (full-screen menus, loading).
    pub bare: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ModalResponse {
    /// Build its content this tick.
    pub visible: bool,
    /// Back, or a press on the backdrop, asked to close it this tick.
    pub dismissed: bool,
}

/// A virtualised list: only the rows in view (and one more each side) are built.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ListDesc {
    pub count: u32,
    /// Every row's height; 0: the density's row height.
    pub row_height: f32,
    pub gap: f32,
    pub width: Size,
    /// The list's height (it scrolls inside); the zero value grows.
    pub height: Option<Size>,
    pub padding: Insets,
}

/// The rows to build this tick: `first..last`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListRange {
    pub first: u32,
    pub last: u32,
}

/// A virtualised grid of equal cells (inventories, level select).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GridDesc {
    pub count: u32,
    /// Cell size; 0: 64 CSS px (and at least the touch target). With
    /// `columns` and a width other than fit, the cells share the grid's
    /// width instead (the height stays).
    pub cell_width: f32,
    pub cell_height: f32,
    /// Cells across, exactly; 0: as many as fit.
    pub columns: u32,
    /// 0: `space.sm`.
    pub gap: f32,
    pub width: Size,
    pub height: Option<Size>,
    pub padding: Insets,
}

/// A segmented control of tabs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabsDesc {
    /// Equal segments across the width.
    pub fill: bool,
    pub width: Option<Size>,
    /// The tab row's accessible name (what a screen reader calls it);
    /// empty is its id.
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectDesc {
    /// Shown while nothing is selected.
    pub placeholder: String,
    pub width: Option<Size>,
    pub disabled: bool,
    /// Its accessible name; empty is its id.
    pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepperDesc {
    /// The range; `min == max` (the zero value): unbounded.
    pub min: f32,
    pub max: f32,
    /// 0: 1.
    pub step: f32,
    /// Digits after the point.
    pub decimals: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RadioDesc {
    pub horizontal: bool,
    /// The group's accessible name; empty is its id.
    pub label: String,
}

/// Status tones (badges, toasts, map markers).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warn,
    Danger,
    Info,
    Primary,
    Inverse,
}

impl Tone {
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Its name as a selector attribute (`Badge.base[tone=danger]`).
    pub fn name(self) -> &'static str {
        [
            "neutral", "success", "warn", "danger", "info", "primary", "inverse",
        ][self as usize]
    }

    pub fn from_index(i: u8) -> Tone {
        [
            Tone::Neutral,
            Tone::Success,
            Tone::Warn,
            Tone::Danger,
            Tone::Info,
            Tone::Primary,
            Tone::Inverse,
        ]
        .get(i as usize)
        .copied()
        .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ToastDesc {
    pub tone: Tone,
    /// 0: 3200 ms (6000 for danger).
    pub duration_ms: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChipDesc {
    pub selected: bool,
    pub disabled: bool,
}

/// What a meter measures, which sets its colour.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MeterKind {
    /// `health`, `warn` under 30 %, `danger` under 15 %.
    #[default]
    Health,
    /// `mana`.
    Mana,
    /// `xp`.
    Xp,
    /// `primary`.
    Primary,
    Custom(Srgba),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeterDesc {
    pub kind: MeterKind,
    /// Text inside the bar (empty: none; the bar grows to fit it).
    pub label: String,
    pub width: Option<Size>,
    /// 0: `space.md`, or the caption's line height with a label.
    pub height: f32,
    /// Notches dividing the bar into this many parts (0, 1: none).
    pub segments: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CooldownDesc {
    /// The diameter; 0: 56 (never under the touch target).
    pub size: f32,
    /// Text in the middle (a key, a count; empty: none).
    pub label: String,
    /// An icon of the atlas in the middle, by name.
    pub icon: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IconDesc {
    /// 0: 24 CSS px.
    pub size: f32,
    /// `None`: `ink`.
    pub tint: Option<Srgba>,
    /// What a screen reader calls it (and the probe's text): `None` is the
    /// icon's name; empty is decoration, named by nothing, so a card it
    /// sits on is named by its other text alone.
    pub label: Option<String>,
}

/// An art slot ([`Ui::art`], docs/ui.md §13).
#[derive(Clone, Debug, PartialEq)]
pub struct ArtDesc {
    pub spec: crate::slot::SlotSpec,
    /// The art once it is there (`None`: the style's placeholder).
    pub texture: Option<TextureHandle>,
    /// The texture's size in pixels, for cover and contain (`None`: the
    /// slot's aspect).
    pub size: Option<[f32; 2]>,
    pub width: Size,
    pub height: Size,
    /// What a screen reader calls it; empty is the slot's name.
    pub label: String,
}

/// A marker on a mini-map, in the map's 0–1 coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapMarker {
    pub x: f32,
    pub y: f32,
    pub tone: Tone,
    /// The player: the accent, larger, ringed.
    pub player: bool,
}

/// A zoomable picture ([`Ui::zoom_view`]): a map, an artwork.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZoomViewDesc {
    /// `None` draws a solid rectangle of `tint`.
    pub texture: Option<TextureHandle>,
    /// The picture: `None` is the whole texture (`[u0, v0, u1, v1]`, v down).
    pub uv: Option<[f32; 4]>,
    /// `None`: white.
    pub tint: Option<Srgba>,
    pub width: Size,
    pub height: Size,
    /// Width over height (0: none), as `ImageDesc::aspect`.
    pub aspect: f32,
    /// The zoom limits; 0: 1 and 4.
    pub min_zoom: f32,
    pub max_zoom: f32,
}

/// Where a zoom view is: its zoom and the middle of what it shows, as a
/// fraction of its picture ((0.5, 0.5) at rest).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoomResponse {
    pub zoom: f32,
    pub center: og_core::glam::Vec2,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MinimapDesc {
    pub texture: Option<TextureHandle>,
    pub uv: Option<[f32; 4]>,
    /// 0: 160 CSS px square.
    pub size: f32,
    pub markers: Vec<MapMarker>,
    /// A label under the frame (a zone name; empty: none).
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogueDesc {
    pub speaker: String,
    pub text: String,
    /// A portrait texture region (`None`: the icon, else the speaker's initial).
    pub portrait: Option<(TextureHandle, [f32; 4])>,
    /// A portrait from the icon atlas by name (when no texture is given).
    pub portrait_icon: String,
    /// Characters per second; 0: 40.
    pub chars_per_second: f32,
    /// The portrait on the right.
    pub portrait_right: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DialogueResponse {
    /// Characters shown.
    pub shown: u32,
    /// The whole line is shown.
    pub done: bool,
    /// Pressed (or activated) while done: show the next line.
    pub advanced: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JoystickDesc {
    /// The base's diameter; 0: 128 (at least three touch targets).
    pub size: f32,
    /// Ignored radius as a ratio; 0: 0.15.
    pub dead_zone: f32,
    /// The player can move, scale and fade it ([`crate::Ui::set_hud_edit`]).
    pub customize: Option<crate::Customize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JoystickResponse {
    /// -1..1 right, -1..1 down; 0 inside the dead zone.
    pub x: f32,
    pub y: f32,
    /// A press that started on it is held.
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DpadDesc {
    /// The whole pad's size; 0: 132 (each arm at least the touch target).
    pub size: f32,
    /// The player can move, scale and fade it ([`crate::Ui::set_hud_edit`]).
    pub customize: Option<crate::Customize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DpadResponse {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionButtonDesc {
    /// 0: 64 (never under the touch target).
    pub size: f32,
    pub primary: bool,
    /// An icon of the atlas by name, instead of the label.
    pub icon: String,
    /// The player can move, scale and fade it ([`crate::Ui::set_hud_edit`]).
    pub customize: Option<crate::Customize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkeletonDesc {
    pub width: Option<Size>,
    /// 0: one body line.
    pub height: f32,
    /// Stacked lines (0, 1: one); the last is shorter.
    pub lines: u32,
}

fn hash_str(s: &str) -> u64 {
    let mut h = StableHasher::new();
    h.write_str(s);
    h.finish()
}

impl Ui {
    /// The touch target floor: `controlSm` of the touch density (44).
    pub fn min_target(&self) -> f32 {
        crate::theme::TOUCH.control_sm
    }

    /// A number the UI keeps for `id` under the open element (a template's
    /// tab, a count-up's start), in the UI state like every widget memory.
    /// Read it every tick it is used: an unused memory is dropped.
    pub fn memory(&mut self, id: &str, initial: f64) -> f64 {
        let key = crate::id::child_key(self.key_for(id), "memory");
        if !self.has_anim(key) {
            self.set_anim(
                key,
                Anim {
                    h: initial.to_bits(),
                    ..Anim::default()
                },
            );
            return initial;
        }
        f64::from_bits(self.anim(key).h)
    }

    /// Sets a number memory (see [`Ui::memory`]).
    pub fn remember(&mut self, id: &str, value: f64) {
        let key = crate::id::child_key(self.key_for(id), "memory");
        self.set_anim(
            key,
            Anim {
                h: value.to_bits(),
                ..Anim::default()
            },
        );
    }
}

fn cos(a: f32) -> f32 {
    a.cos()
}

fn sin(a: f32) -> f32 {
    a.sin()
}

mod hud;
mod layers;
mod lists;
mod presentation;
mod selection;
mod touch_dialogue;
