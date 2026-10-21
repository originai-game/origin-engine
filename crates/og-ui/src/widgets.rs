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
use crate::ui::{Anim, Response, Toast, Ui};
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
    /// The skin frame its background takes (`None`: the panel's).
    pub skin: Option<crate::skin::SkinPart>,
    /// Shown settled the first time it is built: no skin entrance or
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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsDesc {
    /// Equal segments across the width.
    pub fill: bool,
    pub width: Option<Size>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SelectDesc {
    /// Shown while nothing is selected.
    pub placeholder: String,
    pub width: Option<Size>,
    pub disabled: bool,
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadioDesc {
    pub horizontal: bool,
}

/// Status tones (`docs/design-language.md`: Badge, Toast).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Success,
    Warn,
    Danger,
    Info,
    Accent,
    Inverse,
}

impl Tone {
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(i: u8) -> Tone {
        [
            Tone::Neutral,
            Tone::Success,
            Tone::Warn,
            Tone::Danger,
            Tone::Info,
            Tone::Accent,
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
    /// 0: 3200 ms (6000 for danger), as the platform's toasts.
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
    /// Green, warning orange under 30 %, danger red under 15 %.
    #[default]
    Health,
    /// The blue hue.
    Mana,
    /// The purple hue.
    Xp,
    /// The accent.
    Accent,
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
    /// `None`: `fg`.
    pub tint: Option<Srgba>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JoystickDesc {
    /// The base's diameter; 0: 128 (at least three touch targets).
    pub size: f32,
    /// Ignored radius as a ratio; 0: 0.15.
    pub dead_zone: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct JoystickResponse {
    /// -1..1 right, -1..1 down; 0 inside the dead zone.
    pub x: f32,
    pub y: f32,
    /// A press that started on it is held.
    pub active: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DpadDesc {
    /// The whole pad's size; 0: 132 (each arm at least the touch target).
    pub size: f32,
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
    /// The touch target floor: `h-control-sm` of the touch density (44).
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

    // ── layers ─────────────────────────────────────────────────────────────

    /// Opens an anchored group; children follow until [`Ui::close`].
    pub fn open_anchor(&mut self, id: impl Into<Id>, desc: AnchorDesc) {
        let margin = desc.margin.unwrap_or(self.sizes().gutter);
        let el = Element {
            direction: desc.direction,
            width: desc.width,
            height: desc.height,
            padding: desc.padding,
            gap: desc.gap,
            align: desc.align,
            cross: desc.cross,
            background: desc.background,
            radius: desc.radius,
            skin: desc.skin.or(Some(crate::skin::SkinPart::Panel)),
            ..Element::default()
        };
        let i = self.open_layer(
            "anchor",
            id.into(),
            el,
            Float::Anchored {
                anchor: desc.anchor,
                margin,
                dx: desc.offset.0,
                dy: desc.offset.1,
                over: desc.over,
            },
            NONE,
        );
        // A skin's entrance and stagger for a screen panel, from the tick it
        // first appears; an instant panel is settled from there (`h` 1).
        let m = self.theme.skin.motion;
        if desc.background.is_some() && (m.enter.is_some() || m.stagger_ms > 0.0) {
            let key = crate::id::child_key(self.nodes[i].key, "enter");
            let fresh = !self.has_anim(key);
            let mut a = self.anim(key);
            if fresh {
                a.t = self.state.tick;
                a.h = u64::from(desc.instant);
            }
            self.set_anim(key, a);
            if a.h == 0 {
                self.enter(i, a.t, m.enter);
            }
        }
    }

    /// Plays a skin entrance on layer `i` from tick `since`: its own
    /// transition (`kind`) and its controls' stagger.
    fn enter(&mut self, i: usize, since: u64, kind: Option<TransitionKind>) {
        let t = self.theme;
        let ticks = self.ticks_for(t.durations_ms.base) as f32;
        let p = ((self.state.tick - since) as f32 + 1.0) / ticks;
        if let Some(kind) = kind
            && p < 1.0
        {
            let mut f = fx(kind, ease_out(p.min(1.0)), t.space.xl, t.reduced_motion);
            f.inert = true;
            self.nodes[i].fx = Some(f);
        }
        let stagger = t.skin.motion.stagger_ms;
        if stagger > 0.0 && !t.reduced_motion {
            self.nodes[i].stagger = Some(crate::layout::Stagger {
                since,
                step: stagger * self.ticks_per_second as f32 / 1000.0,
                ticks,
                kind: TransitionKind::SlideUp,
            });
        }
    }

    /// An anchored group built by `f`.
    pub fn anchor(&mut self, id: impl Into<Id>, desc: AnchorDesc, f: impl FnOnce(&mut Ui)) {
        self.open_anchor(id, desc);
        f(self);
        self.close();
    }

    /// Advances a presence memory and answers it with its eased value.
    fn presence(&mut self, key: u64, show: bool, ms: f32, instant: bool) -> (Presence, f32) {
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        if fresh {
            a.a = if show && instant { 1.0 } else { 0.0 };
        }
        let step = 1.0 / self.ticks_for(ms) as f32;
        let before = a.a;
        a.a = if show {
            (a.a + step).min(1.0)
        } else {
            (a.a - step).max(0.0)
        };
        if fresh && show && !instant {
            a.a = step.min(1.0);
        }
        self.set_anim(key, a);
        let p = a.a;
        let visible = p > 0.0 || show;
        let e = if show {
            ease_out(p)
        } else {
            1.0 - ease_exit(1.0 - p)
        };
        (
            Presence {
                visible,
                progress: p,
                entering: show && p < 1.0,
                leaving: !show && p > 0.0 && before > 0.0,
            },
            e,
        )
    }

    /// Opens a transition container (a column); children follow until
    /// [`Ui::close`]. Build them only while `visible`: it stays visible while
    /// it leaves. Interaction inside waits until it settles.
    pub fn open_transition(
        &mut self,
        id: impl Into<Id>,
        show: bool,
        desc: TransitionDesc,
    ) -> Presence {
        let id = id.into();
        let i = self.push(
            "transition",
            id,
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
        );
        self.stack.push(i);
        let key = self.nodes[i].key;
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else {
            self.theme.durations_ms.base
        };
        let (pr, e) = self.presence(key, show, ms, desc.instant);
        let distance = if desc.distance > 0.0 {
            desc.distance
        } else {
            self.theme.space.xl
        };
        let mut f = fx(desc.kind, e, distance, self.theme.reduced_motion);
        f.inert = pr.progress < 1.0 || !show;
        self.nodes[i].fx = Some(f);
        self.nodes[i].hidden = !pr.visible;
        pr
    }

    /// A transition built by `f` while visible.
    pub fn transition(
        &mut self,
        id: impl Into<Id>,
        show: bool,
        desc: TransitionDesc,
        f: impl FnOnce(&mut Ui),
    ) -> Presence {
        let p = self.open_transition(id, show, desc);
        if p.visible {
            f(self);
        }
        self.close();
        p
    }

    /// Opens a dialog: a scrim backdrop over everything with a centred
    /// panel (`surface-raised`, `radius.panel`). Its content follows until
    /// [`Ui::close`]; build it only while `visible`. Focus moves into it and
    /// stays there while it is the topmost; Back and the backdrop dismiss it
    /// unless `locked`. The probe sees the backdrop as `<id>` and the panel
    /// as `<id>/Panel`.
    pub fn open_modal(&mut self, id: impl Into<Id>, open: bool, desc: &ModalDesc) -> ModalResponse {
        let t = self.theme;
        let backdrop = self.open_layer(
            "modal",
            id.into(),
            Element {
                direction: Direction::Column,
                align: Align::Center,
                cross: Cross::Center,
                padding: Insets::all(self.sizes().gutter),
                background: Some(desc.backdrop.unwrap_or(t.palette.scrim)),
                ..Element::default()
            },
            Float::Modal,
            NONE,
        );
        let key = self.nodes[backdrop].key;
        let (pr, e) = self.presence(key, open, t.durations_ms.base, false);
        if pr.entering && pr.progress <= 1.0 / self.ticks_for(t.durations_ms.base) as f32 + 1e-6 {
            self.sound(crate::skin::UiSound::Open);
        }
        if pr.leaving
            && pr.progress >= 1.0 - 1.0 / self.ticks_for(t.durations_ms.base) as f32 - 1e-6
        {
            self.sound(crate::skin::UiSound::Close);
        }
        let kind = desc
            .transition
            .or(t.skin.motion.enter)
            .unwrap_or(TransitionKind::Scale);
        let mut r = ModalResponse {
            visible: pr.visible,
            dismissed: false,
        };
        {
            let n = &mut self.nodes[backdrop];
            n.hidden = !pr.visible;
            n.backdrop = true;
            n.scope = true;
            n.fx = Some(crate::motion::Fx {
                alpha: e,
                inert: pr.progress < 1.0 || !open,
                ..Default::default()
            });
        }
        if open && !desc.locked {
            let resp = self.respond(key);
            if (resp.clicked && !desc.ignore_backdrop) || (self.state.scope == key && self.nav.back)
            {
                if self.state.scope == key {
                    self.nav.back = false;
                }
                r.dismissed = true;
            }
        }
        let vw = self.viewport.width - self.viewport.safe_area.horizontal();
        let width = desc
            .width
            .unwrap_or(Size::Fixed(420.0f32.min(vw - 2.0 * self.sizes().gutter)));
        let panel = self.push(
            "panel",
            Id::from("Panel"),
            Element {
                direction: Direction::Column,
                width,
                padding: if desc.bare {
                    Insets::default()
                } else {
                    Insets::all(t.space.xl)
                },
                gap: t.space.md,
                background: (!desc.bare).then_some(t.palette.surface_raised),
                radius: t.radius.panel,
                skin: Some(crate::skin::SkinPart::Panel),
                interactive: !desc.bare,
                cross: if desc.bare {
                    Cross::Center
                } else {
                    Cross::Start
                },
                ..Element::default()
            },
        );
        self.stack.push(panel);
        self.nodes[panel].a11y.role = Some("dialog");
        self.nodes[panel].a11y.name = Some(desc.title.clone());
        if open && t.skin.motion.stagger_ms > 0.0 {
            let since = self.since(crate::id::child_key(key, "enter"));
            self.enter(panel, since, None);
        }
        let moving = fx(kind, e, t.space.xl, t.reduced_motion);
        self.nodes[panel].fx = Some(crate::motion::Fx {
            alpha: 1.0,
            inert: false,
            ..moving
        });
        if !desc.title.is_empty() && pr.visible {
            self.label(
                "Title",
                &desc.title,
                crate::LabelDesc {
                    step: Some(t.type_ramp.xl),
                    heading: true,
                    ..Default::default()
                },
            );
        }
        r
    }

    /// Closes a dialog opened with [`Ui::open_modal`] (its panel and backdrop).
    pub fn close_modal(&mut self) {
        self.close();
        self.close();
    }

    /// A dialog built by `f` while visible.
    pub fn modal(
        &mut self,
        id: impl Into<Id>,
        open: bool,
        desc: &ModalDesc,
        f: impl FnOnce(&mut Ui),
    ) -> ModalResponse {
        let r = self.open_modal(id, open, desc);
        if r.visible {
            f(self);
        }
        self.close_modal();
        r
    }

    /// Queues a toast: it stacks at the toast anchor, enters, stays for its
    /// duration and leaves; at most three are up (older ones leave first).
    /// A press on one dismisses it. Callable between ticks. Returns its id.
    pub fn toast(&mut self, text: &str, desc: ToastDesc) -> u32 {
        let ms = if desc.duration_ms > 0.0 {
            desc.duration_ms
        } else if desc.tone == Tone::Danger {
            6000.0
        } else {
            3200.0
        };
        self.state.next_toast += 1;
        let id = self.state.next_toast;
        let born = self.state.tick + u64::from(!self.building);
        let ticks = self.ticks_for(ms);
        match desc.tone {
            Tone::Success => self.sound(crate::skin::UiSound::Success),
            Tone::Danger => self.sound(crate::skin::UiSound::Error),
            _ => {}
        }
        self.state.toasts.push(Toast {
            id,
            text: text.to_owned(),
            tone: desc.tone.index(),
            born,
            ticks,
            leaving: 0,
        });
        id
    }

    /// Starts a toast's exit.
    pub fn dismiss_toast(&mut self, id: u32) {
        let now = self.state.tick;
        for t in self.state.toasts.iter_mut() {
            if t.id == id && t.leaving == 0 {
                t.leaving = now.max(1);
            }
        }
    }

    /// The toasts up now (id, text), oldest first.
    pub fn toasts(&self) -> Vec<(u32, String)> {
        self.state
            .toasts
            .iter()
            .map(|t| (t.id, t.text.clone()))
            .collect()
    }

    /// Builds the toast stack (called by `end`).
    pub(crate) fn build_toasts(&mut self) {
        let now = self.state.tick;
        let exit = u64::from(self.ticks_for(self.theme.durations_ms.fast));
        let enter = self.ticks_for(self.theme.durations_ms.base);
        // Expire, cap at three, drop the ones whose exit finished.
        let mut up: Vec<usize> = Vec::new();
        for (k, t) in self.state.toasts.iter_mut().enumerate() {
            if t.leaving == 0 && now >= t.born + u64::from(t.ticks) {
                t.leaving = now;
            }
            if t.leaving == 0 {
                up.push(k);
            }
        }
        if up.len() > 3 {
            for &k in &up[..up.len() - 3] {
                self.state.toasts[k].leaving = now;
            }
        }
        self.state
            .toasts
            .retain(|t| t.leaving == 0 || now < t.leaving + exit);
        if self.state.toasts.is_empty() {
            return;
        }
        let t = self.theme;
        let top = matches!(
            self.toast_anchor,
            Anchor::Top | Anchor::TopLeft | Anchor::TopRight
        );
        let margin = self.sizes().gutter;
        self.open_layer(
            "toasts",
            Id::from("Toasts"),
            Element {
                direction: Direction::Column,
                gap: t.space.sm,
                cross: Cross::Center,
                ..Element::default()
            },
            Float::Toasts {
                anchor: self.toast_anchor,
                margin,
            },
            NONE,
        );
        let toasts = self.state.toasts.clone();
        let mut dismiss = Vec::new();
        for toast in toasts {
            let age = now.saturating_sub(toast.born) as f32;
            let p = if toast.leaving != 0 {
                1.0 - ((now - toast.leaving) as f32 / exit as f32).min(1.0)
            } else {
                ((age + 1.0) / enter as f32).min(1.0)
            };
            let e = if toast.leaving != 0 {
                1.0 - ease_exit(1.0 - p)
            } else {
                ease_out(p)
            };
            let kind = t.skin.motion.toast.unwrap_or(if top {
                TransitionKind::SlideDown
            } else {
                TransitionKind::SlideUp
            });
            let distance = match kind {
                TransitionKind::SlideLeft | TransitionKind::SlideRight => 4.0 * t.space.xxl,
                _ => t.space.lg,
            };
            let i = self.push(
                "toast",
                crate::ui_idi("Toast", toast.id),
                Element {
                    direction: Direction::Row,
                    height: Size::Fixed(self.sizes().control_lg),
                    padding: Insets::xy(t.space.lg, 0.0),
                    gap: t.space.sm,
                    cross: Cross::Center,
                    background: Some(t.palette.surface_raised),
                    radius: t.radius.button,
                    interactive: true,
                    skin: Some(crate::skin::SkinPart::Capsule),
                    ..Element::default()
                },
            );
            self.stack.push(i);
            let mut f = fx(kind, e, distance, t.reduced_motion);
            f.inert = toast.leaving != 0;
            self.nodes[i].fx = Some(f);
            if self.respond(self.nodes[i].key).clicked {
                dismiss.push(toast.id);
            }
            let tone = Tone::from_index(toast.tone);
            if tone != Tone::Neutral {
                let dot = t.space.sm;
                let c = self.tone_dot(tone);
                self.nodes[i].shapes.push(Shape::Rect {
                    frame: Frame {
                        fy: 0.5,
                        x: t.space.lg,
                        y: -dot / 2.0,
                        w: dot,
                        h: dot,
                        ..Frame::default()
                    },
                    radius: dot,
                    color: c,
                });
                self.nodes[i].el.padding.left = t.space.lg + dot + t.space.sm;
            }
            self.label(
                "Text",
                &toast.text,
                crate::LabelDesc {
                    nowrap: true,
                    ..Default::default()
                },
            );
            self.nodes[i].probe_text = toast.text.clone();
            self.close();
        }
        self.close();
        for id in dismiss {
            self.dismiss_toast(id);
        }
    }

    fn tone_dot(&self, tone: Tone) -> Srgba {
        let p = &self.theme.palette;
        match tone {
            Tone::Success => p.success,
            Tone::Warn => p.warn,
            Tone::Danger => p.danger,
            Tone::Info => p.info,
            Tone::Accent => p.accent,
            Tone::Inverse => p.fg,
            Tone::Neutral => p.fg_muted,
        }
    }

    /// A spotlight over everything (a tutorial step): the scrim on four
    /// sides of `hole` (CSS px; `None`: the whole surface dimmed), an amber
    /// edge around it. Presses on the dimmed parts stop there (`clicked`);
    /// presses in the hole reach what is under it. Focus stays with the
    /// controls under the hole and the layers above it. The probe sees the
    /// shades as `<id>/Top`, `Left`, `Right`, `Bottom` and the hole as `<id>/Hole`.
    pub fn spotlight(
        &mut self,
        id: impl Into<Id>,
        hole: Option<crate::Rect>,
        desc: SpotlightDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let vp = self.viewport;
        let pad = if desc.padding > 0.0 {
            desc.padding
        } else {
            t.space.sm
        };
        let h = hole.map(|r| r.outset(pad)).map(|r| {
            let x = r.x.clamp(0.0, vp.width);
            let y = r.y.clamp(0.0, vp.height);
            crate::Rect::new(
                x,
                y,
                (r.right().min(vp.width) - x).max(0.0),
                (r.bottom().min(vp.height) - y).max(0.0),
            )
        });
        let layer = self.open_layer(
            "spotlight",
            id.into(),
            Element {
                direction: Direction::Column,
                ..Element::default()
            },
            Float::Modal,
            NONE,
        );
        // The modal placement pads by the safe area; the shades must not.
        self.nodes[layer].el.padding = Insets::default();
        let mut out = Response::default();
        let mut shade = |ui: &mut Ui, name: &str, w: Size, hgt: Size| {
            let i = ui.leaf(
                "shade",
                Id::from(name),
                Element {
                    width: w,
                    height: hgt,
                    background: Some(p.scrim.fade(1.6)),
                    interactive: true,
                    ..Element::default()
                },
            );
            let r = ui.respond(ui.nodes[i].key);
            out.clicked |= r.clicked;
            out.hovered |= r.hovered;
        };
        match h {
            None => shade(self, "Top", Size::grow(), Size::grow()),
            Some(h) => {
                shade(self, "Top", Size::grow(), Size::Fixed(h.y));
                self.open(
                    "Middle",
                    Element {
                        direction: Direction::Row,
                        width: Size::grow(),
                        height: Size::Fixed(h.height),
                        ..Element::default()
                    },
                );
                shade(self, "Left", Size::Fixed(h.x), Size::grow());
                let hole = self.leaf(
                    "hole",
                    Id::from("Hole"),
                    Element {
                        width: Size::Fixed(h.width),
                        height: Size::grow(),
                        ..Element::default()
                    },
                );
                let w = t.space.xxs;
                let edges = [
                    Frame {
                        fw: 1.0,
                        h: w,
                        ..Frame::default()
                    },
                    Frame {
                        fw: 1.0,
                        fy: 1.0,
                        y: -w,
                        h: w,
                        ..Frame::default()
                    },
                    Frame {
                        fh: 1.0,
                        w,
                        ..Frame::default()
                    },
                    Frame {
                        fx: 1.0,
                        x: -w,
                        fh: 1.0,
                        w,
                        ..Frame::default()
                    },
                ];
                for frame in edges {
                    self.nodes[hole].shapes.push(Shape::Rect {
                        frame,
                        radius: 0.0,
                        color: p.focus,
                    });
                }
                shade(self, "Right", Size::grow(), Size::grow());
                self.close();
                shade(self, "Bottom", Size::grow(), Size::grow());
            }
        }
        self.close();
        out
    }

    // ── lists and grids ────────────────────────────────────────────────────

    /// The visible height of a scroll container last frame, or a guess.
    fn view_height(&self, key: u64, size: Option<Size>) -> f32 {
        if let Some(h) = self.hit(key) {
            return h.rect.height;
        }
        match size {
            Some(Size::Fixed(v)) => v,
            _ => self.viewport.height,
        }
    }

    /// Opens a virtualised list (a scroll column) and answers which rows to
    /// build: call [`Ui::list_item`] for each of `first..last`, then
    /// [`Ui::close`]. Fillers keep the scroll range of all `count` rows.
    pub fn open_list(&mut self, id: impl Into<Id>, desc: ListDesc) -> ListRange {
        let id = id.into();
        let row = if desc.row_height > 0.0 {
            desc.row_height
        } else {
            self.sizes().row
        };
        let key = self.key_for(id.as_str());
        let view = self.view_height(key, desc.height) - desc.padding.vertical();
        let off = self.state.scroll.get(&key).copied().unwrap_or(0.0);
        let stride = row + desc.gap;
        let count = desc.count;
        let first = ((off / stride).floor().max(0.0) as u32)
            .saturating_sub(1)
            .min(count);
        let last = ((((off + view) / stride).ceil().max(0.0)) as u32 + 1).min(count);
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width,
                height: desc.height.unwrap_or(Size::grow()),
                padding: desc.padding,
                gap: desc.gap,
                cross: Cross::Stretch,
                scroll: true,
                ..Element::default()
            },
        );
        let list = self.parent();
        self.nodes[list].aux = [row, 0.0, -1.0];
        if first > 0 {
            self.filler(first as f32 * stride - desc.gap, Size::grow());
        }
        // The bottom filler goes in when the list closes.
        let rest = (count - last) as f32 * stride - desc.gap;
        self.nodes[list].aux[2] = if count > last { rest } else { -1.0 };
        ListRange { first, last }
    }

    fn filler(&mut self, h: f32, w: Size) {
        let i = self.leaf(
            "spacer",
            Id::default(),
            Element {
                width: w,
                height: Size::Fixed(h.max(0.0)),
                ..Element::default()
            },
        );
        self.nodes[i].widget = Widget::Spacer;
    }

    /// Closes a list or grid opened with [`Ui::open_list`] / [`Ui::open_grid`].
    pub fn close_list(&mut self) -> Response {
        let list = self.parent();
        let rest = self.nodes[list].aux[2];
        if rest >= 0.0 {
            let w = if self.nodes[list].el.wrap {
                Size::Percent(1.0)
            } else {
                Size::grow()
            };
            self.filler(rest, w);
        }
        self.close()
    }

    /// Opens row `index` of the open list (`Row#index`): a selectable,
    /// focusable row; its content follows until [`Ui::close`].
    pub fn list_item(&mut self, index: u32, selected: bool) -> Response {
        let t = self.theme;
        let list = self.parent();
        let h = self.nodes[list].aux[0];
        let i = self.push(
            "row",
            crate::ui_idi("Row", index),
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                height: Size::Fixed(h),
                padding: Insets::xy(t.space.md, 0.0),
                gap: t.space.sm,
                cross: Cross::Center,
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        self.stack.push(i);
        self.nodes[i].a11y.role = Some("option");
        self.nodes[i].a11y.selected = Some(selected);
        let mut r = self.nav_target(i, false);
        r.dropped = self.drag_and_drop(list, i, index);
        let p = &t.palette;
        self.nodes[i].el.background = if selected {
            Some(p.fill_strong)
        } else if r.hovered || r.pressed {
            Some(p.fill)
        } else {
            None
        };
        r
    }

    /// Drag and drop between the items of one list or grid: a press on an
    /// item marks it the source (in the container's memory); a release over
    /// another item answers the source's index there.
    fn drag_and_drop(&mut self, container: usize, item: usize, index: u32) -> Option<u32> {
        let ckey = self.nodes[container].key;
        let key = self.nodes[item].key;
        let mut a = self.anim(ckey);
        let mut dropped = None;
        if self.input.pressed && self.state.active == key {
            a.h = u64::from(index) + 1;
        } else if self.input.released && a.h > 0 {
            if self.state.hot == key && self.state.active != key && self.state.active != 0 {
                dropped = Some((a.h - 1) as u32);
            }
        } else if !self.input.down && !self.input.released {
            a.h = 0;
        }
        self.set_anim(ckey, a);
        dropped
    }

    /// Whether a drag from another item of `container` is over `item` now.
    fn drop_target(&self, container: usize, item: usize) -> bool {
        let ckey = self.nodes[container].key;
        let key = self.nodes[item].key;
        let a = self.state.anims.get(&ckey).copied().unwrap_or_default();
        a.h > 0
            && self.input.down
            && self.state.hot == key
            && self.state.active != key
            && self.state.active != 0
    }

    /// The index of the list or grid item a held press started on (a drag
    /// in progress), for drawing what is dragged under the pointer.
    pub fn dragging(&mut self, id: impl Into<Id>) -> Option<u32> {
        let id = id.into();
        let key = self.key_for(id.as_str());
        let a = self.state.anims.get(&key).copied().unwrap_or_default();
        (self.input.down && a.h > 0 && self.state.active != 0).then(|| (a.h - 1) as u32)
    }

    /// A virtualised list built by `row(ui, index)` for the rows in view;
    /// answers the row clicked this tick.
    pub fn list(
        &mut self,
        id: impl Into<Id>,
        desc: ListDesc,
        selected: Option<u32>,
        mut row: impl FnMut(&mut Ui, u32),
    ) -> Option<u32> {
        let range = self.open_list(id, desc);
        let mut clicked = None;
        for k in range.first..range.last {
            if self.list_item(k, selected == Some(k)).clicked {
                clicked = Some(k);
            }
            row(self, k);
            self.close();
        }
        self.close_list();
        clicked
    }

    /// Opens a virtualised grid (a wrapping, scrolling row of equal cells):
    /// call [`Ui::grid_item`] for each of `first..last`, then [`Ui::close_list`].
    pub fn open_grid(&mut self, id: impl Into<Id>, desc: GridDesc) -> ListRange {
        let id = id.into();
        let t = self.theme;
        let floor = self.min_target();
        let asked = if desc.cell_width > 0.0 {
            desc.cell_width
        } else {
            64.0
        }
        .max(floor);
        let ch = if desc.cell_height > 0.0 {
            desc.cell_height
        } else {
            asked
        }
        .max(floor);
        let gap = if desc.gap > 0.0 { desc.gap } else { t.space.sm };
        let key = self.key_for(id.as_str());
        // The width the cells had last frame (the grid's, inside its padding).
        let known_w = self
            .hit(key)
            .map(|h| h.rect.width - desc.padding.horizontal());
        // `columns` across, exactly: a fit grid is that many cells of the
        // asked width; any other width is shared by them (wider cells, or
        // narrower down to the touch target), rounded down to 1/64 px so a
        // row never spills into the next. Else as many as fit.
        let (columns, cw) = if desc.columns > 0 {
            let c = desc.columns;
            let cw = match known_w {
                Some(w) if !matches!(desc.width, Size::Fit { .. }) => {
                    (((w + gap) / c as f32 - gap) * 64.0).floor() / 64.0
                }
                _ => asked,
            };
            (c, cw.max(floor))
        } else {
            let inner_w = known_w.unwrap_or(self.viewport.width - desc.padding.horizontal());
            (
                (((inner_w + gap) / (asked + gap)).floor() as u32).max(1),
                asked,
            )
        };
        let view = self.view_height(key, desc.height) - desc.padding.vertical();
        let off = self.state.scroll.get(&key).copied().unwrap_or(0.0);
        let stride = ch + gap;
        let rows = desc.count.div_ceil(columns);
        let first_row = ((off / stride).floor().max(0.0) as u32)
            .saturating_sub(1)
            .min(rows);
        let last_row = ((((off + view) / stride).ceil().max(0.0)) as u32 + 1).min(rows);
        let width = if desc.columns > 0 && matches!(desc.width, Size::Fit { .. }) {
            Size::Fixed(columns as f32 * stride - gap + desc.padding.horizontal())
        } else {
            desc.width
        };
        self.open(
            id,
            Element {
                direction: Direction::Row,
                wrap: true,
                scroll: true,
                width,
                height: desc.height.unwrap_or(Size::grow()),
                padding: desc.padding,
                gap,
                ..Element::default()
            },
        );
        let grid = self.parent();
        self.nodes[grid].aux = [cw, ch, -1.0];
        if first_row > 0 {
            self.filler(first_row as f32 * stride - gap, Size::Percent(1.0));
        }
        let first = (first_row * columns).min(desc.count);
        let last = (last_row * columns).min(desc.count);
        let rest_rows = rows - last_row;
        self.nodes[grid].aux[2] = if rest_rows > 0 {
            rest_rows as f32 * stride - gap
        } else {
            -1.0
        };
        ListRange { first, last }
    }

    /// Opens cell `index` of the open grid (`Cell#index`): a slot on `fill`
    /// (`fill-strong` and the accent ring when selected), focusable; its
    /// content follows until [`Ui::close`].
    pub fn grid_item(&mut self, index: u32, selected: bool) -> Response {
        let t = self.theme;
        let grid = self.parent();
        let [cw, ch, _] = self.nodes[grid].aux;
        let i = self.push(
            "cell",
            crate::ui_idi("Cell", index),
            Element {
                direction: Direction::Column,
                width: Size::Fixed(cw),
                height: Size::Fixed(ch),
                align: Align::Center,
                cross: Cross::Center,
                padding: Insets::all(t.space.xs),
                interactive: true,
                radius: t.radius.block,
                skin: Some(crate::skin::SkinPart::Slot),
                ..Element::default()
            },
        );
        self.stack.push(i);
        self.nodes[i].a11y.role = Some("option");
        self.nodes[i].a11y.selected = Some(selected);
        let mut r = self.nav_target(i, false);
        r.dropped = self.drag_and_drop(grid, i, index);
        let p = &t.palette;
        self.nodes[i].el.background = Some(if selected || r.pressed {
            p.fill_strong
        } else if r.hovered {
            p.surface_hover.fade(0.6).over(p.fill.over(p.surface))
        } else {
            p.fill
        });
        if self.drop_target(grid, i) {
            self.nodes[i].ring = true;
        }
        if selected {
            let w = t.space.xxs;
            self.nodes[i].shapes.push(Shape::Rect {
                frame: Frame {
                    fw: 1.0,
                    h: w,
                    fy: 1.0,
                    y: -w - t.space.xs,
                    x: t.space.md,
                    w: -2.0 * t.space.md,
                    ..Frame::default()
                },
                radius: w,
                color: p.accent,
            });
        }
        r
    }

    /// A virtualised grid built by `cell(ui, index)`; answers the cell clicked.
    pub fn grid(
        &mut self,
        id: impl Into<Id>,
        desc: GridDesc,
        selected: Option<u32>,
        mut cell: impl FnMut(&mut Ui, u32),
    ) -> Option<u32> {
        let range = self.open_grid(id, desc);
        let mut clicked = None;
        for k in range.first..range.last {
            if self.grid_item(k, selected == Some(k)).clicked {
                clicked = Some(k);
            }
            cell(self, k);
            self.close();
        }
        self.close_list();
        clicked
    }

    // ── selection ──────────────────────────────────────────────────────────

    /// A segmented control: a neutral track, the selected tab on `thumb`.
    /// Tabs are `Tab#i` (the probe reads their labels); a click or Activate
    /// selects. `changed` when the selection moved.
    pub fn tabs(
        &mut self,
        id: impl Into<Id>,
        items: &[&str],
        selected: &mut usize,
        desc: TabsDesc,
    ) -> Response {
        let t = self.theme;
        let h = self.sizes().control_md;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: desc
                    .width
                    .unwrap_or(if desc.fill { Size::grow() } else { Size::fit() }),
                height: Size::Fixed(h),
                padding: Insets::all(t.space.xxs),
                gap: t.space.xxs,
                background: Some(t.palette.fill),
                radius: t.radius.button,
                ..Element::default()
            },
        );
        let list = self.parent();
        self.nodes[list].a11y.role = Some("tablist");
        let mut out = Response::default();
        for (k, label) in items.iter().enumerate() {
            let i = self.leaf(
                "tab",
                crate::ui_idi("Tab", k as u32),
                Element {
                    width: if desc.fill { Size::grow() } else { Size::fit() },
                    height: Size::grow(),
                    padding: Insets::xy(t.space.md, 0.0),
                    interactive: true,
                    radius: t.radius.button,
                    ..Element::default()
                },
            );
            let r = self.nav_target(i, false);
            let on = *selected == k;
            if r.clicked && !on {
                *selected = k;
                out.changed = true;
            }
            out.hovered |= r.hovered;
            out.focused |= r.focused;
            out.clicked |= r.clicked;
            let on = *selected == k;
            let p = &t.palette;
            let mut c =
                self.text_content(label, t.type_ramp.body, if on { p.fg } else { p.fg_muted });
            c.align = TextAlign::Center;
            c.wrap = false;
            let n = &mut self.nodes[i];
            n.el.background = if on {
                Some(p.thumb)
            } else if r.hovered {
                Some(p.fill)
            } else {
                None
            };
            n.content = Content::Text(c);
            n.probe_text = (*label).to_owned();
            n.a11y.selected = Some(on);
        }
        self.close();
        out
    }

    /// A select: a field-like trigger with the chosen label and a chevron;
    /// a click or Activate opens its menu (`<id>.menu`, options `Option#i`)
    /// under it, a focus scope; choosing closes it with `changed`. Back or a
    /// press outside closes it unchanged.
    pub fn select(
        &mut self,
        id: impl Into<Id>,
        options: &[&str],
        selected: &mut usize,
        desc: &SelectDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let id = id.into();
        let name = id.as_str().to_owned();
        let h = self.sizes().control_md;
        let i = self.leaf(
            "select",
            id,
            Element {
                width: desc.width.unwrap_or(Size::Grow {
                    min: 160.0,
                    max: f32::INFINITY,
                }),
                height: Size::Fixed(h),
                padding: Insets {
                    left: t.space.md,
                    right: t.space.md + t.space.lg,
                    ..Insets::default()
                },
                interactive: !desc.disabled,
                radius: t.radius.field,
                background: Some(p.fill),
                ..Element::default()
            },
        );
        self.nodes[i].disabled = desc.disabled;
        let key = self.nodes[i].key;
        let mut r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        let mut a = self.anim(key);
        let menu_key = crate::id::child_key(self.nodes[0].key, &format!("{name}.menu"));
        let mut open = a.h == 1;
        if r.clicked {
            open = !open;
            if open {
                self.request_focus =
                    crate::id::child_key(menu_key, &format!("Option#{}", *selected));
            }
        }
        // A press outside the trigger and its menu closes it.
        if open && self.input.pressed && !r.clicked {
            let inside = self
                .hit(self.state.hot)
                .is_some_and(|h| h.scope == menu_key || h.key == menu_key)
                || self.state.hot == key;
            if !inside {
                open = false;
            }
        }
        if open && self.state.scope == menu_key && self.nav.back {
            self.nav.back = false;
            open = false;
        }
        let shown = options.get(*selected).copied().unwrap_or("");
        let text = if shown.is_empty() {
            desc.placeholder.as_str()
        } else {
            shown
        };
        let mut c = self.text_content(
            text,
            t.type_ramp.body,
            if shown.is_empty() || desc.disabled {
                p.fg_subtle
            } else {
                p.fg
            },
        );
        c.wrap = false;
        {
            let n = &mut self.nodes[i];
            n.content = Content::Text(c);
            n.probe_text = text.to_owned();
            n.a11y.name = Some(name.clone());
            n.a11y.value = Some(shown.to_owned());
            n.a11y.expanded = Some(open);
            if r.hovered && !desc.disabled {
                n.el.background = Some(p.fill_strong);
            }
            // The chevron: two short bars.
            let ink = if desc.disabled {
                p.fg_subtle
            } else {
                p.fg_muted
            };
            let (len, w) = (t.space.sm, 1.5);
            let flip = if open { -1.0 } else { 1.0 };
            for side in [-1.0f32, 1.0] {
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fx: 1.0,
                        fy: 0.5,
                        x: -t.space.md - t.space.sm / 2.0 + side * len * 0.35 - len / 2.0,
                        y: -w / 2.0,
                        w: len,
                        h: w,
                        ..Frame::default()
                    },
                    angle: side * flip * -FRAC_PI_4,
                    color: ink,
                });
            }
        }
        if open {
            let mut chosen = None;
            self.open_layer(
                "menu",
                Id::from(format!("{name}.menu")),
                Element {
                    direction: Direction::Column,
                    padding: Insets::all(t.space.xs),
                    gap: t.space.xxs,
                    background: Some(p.surface_raised),
                    radius: t.radius.block + t.space.xs,
                    cross: Cross::Stretch,
                    ..Element::default()
                },
                Float::Popover,
                i,
            );
            let menu = self.parent();
            self.nodes[menu].scope = true;
            self.nodes[menu].a11y.role = Some("listbox");
            self.nodes[menu].el.width = Size::Fit {
                min: self.hit(key).map_or(160.0, |h| h.rect.width),
                max: f32::INFINITY,
            };
            for (k, label) in options.iter().enumerate() {
                let o = self.leaf(
                    "option",
                    crate::ui_idi("Option", k as u32),
                    Element {
                        width: Size::grow(),
                        height: Size::Fixed(self.sizes().control_md),
                        padding: Insets::xy(t.space.md, 0.0),
                        interactive: true,
                        radius: t.radius.block,
                        ..Element::default()
                    },
                );
                let or = self.nav_target(o, false);
                if or.clicked {
                    chosen = Some(k);
                }
                let mut c = self.text_content(label, t.type_ramp.body, p.fg);
                c.wrap = false;
                let n = &mut self.nodes[o];
                n.content = Content::Text(c);
                n.probe_text = (*label).to_owned();
                n.a11y.selected = Some(k == *selected);
                n.el.background = if k == *selected || or.hovered {
                    Some(p.fill_strong)
                } else {
                    None
                };
            }
            self.close();
            if let Some(k) = chosen {
                if k != *selected {
                    *selected = k;
                    r.changed = true;
                }
                open = false;
                self.request_focus = key;
            }
        }
        a.h = u64::from(open);
        self.set_anim(key, a);
        r
    }

    /// A number stepper: `−`, the value, `+` (`<id>/Dec`, `<id>/Value`,
    /// `<id>/Inc`); the buttons step by `step` within the range, Left and
    /// Right step it while a button is focused, and a screen reader's
    /// increment and decrement on its spinbutton (`<id>`) step it too.
    pub fn stepper(&mut self, id: impl Into<Id>, value: &mut f32, desc: StepperDesc) -> Response {
        let t = self.theme;
        let step = if desc.step > 0.0 { desc.step } else { 1.0 };
        let bounded = desc.min != desc.max;
        let (lo, hi) = (desc.min.min(desc.max), desc.min.max(desc.max));
        let h = self.sizes().control_md;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                height: Size::Fixed(h),
                gap: t.space.xs,
                cross: Cross::Center,
                ..Element::default()
            },
        );
        let spin = self.parent();
        self.nodes[spin].a11y.role = Some("spinbutton");
        self.nodes[spin].a11y.value = Some(format!("{:.*}", desc.decimals as usize, *value));
        let mut out = Response::default();
        let spin_key = self.nodes[spin].key;
        let mut delta = if self.nav.spin.0 == spin_key {
            self.nav.spin.1
        } else {
            0
        };
        let digits = desc.decimals as usize;
        for (name, sign) in [("Dec", -1), ("Inc", 1)] {
            if name == "Inc" {
                let text = format!("{:.*}", digits, *value);
                self.label(
                    "Value",
                    &text,
                    crate::LabelDesc {
                        align: TextAlign::Center,
                        nowrap: true,
                        font: FontRole::Mono,
                        width: Size::Fit {
                            min: h * 1.25,
                            max: f32::INFINITY,
                        },
                        ..Default::default()
                    },
                );
            }
            let at_end = bounded && if sign < 0 { *value <= lo } else { *value >= hi };
            let i = self.leaf(
                "button",
                Id::from(name),
                Element {
                    width: Size::Fixed(h),
                    height: Size::Fixed(h),
                    interactive: !at_end,
                    radius: t.radius.button,
                    background: Some(t.palette.fill),
                    ..Element::default()
                },
            );
            let r = if at_end {
                Response::default()
            } else {
                self.nav_target(i, true)
            };
            if r.clicked {
                delta += sign;
            }
            if r.focused {
                delta += self.nav.adjust;
            }
            out.hovered |= r.hovered;
            out.focused |= r.focused;
            out.clicked |= r.clicked;
            let p = t.palette;
            let ink = if at_end { p.fg_subtle } else { p.fg };
            let n = &mut self.nodes[i];
            if r.hovered || r.pressed {
                n.el.background = Some(p.fill_strong);
            }
            n.probe_text = if sign < 0 { "−".into() } else { "+".into() };
            n.a11y.name = Some(if sign < 0 { "Decrease" } else { "Increase" }.into());
            let len = t.space.md;
            n.shapes.push(Shape::Rect {
                frame: Frame::centered(0.0, 0.0, len, 2.0),
                radius: 1.0,
                color: ink,
            });
            if sign > 0 {
                n.shapes.push(Shape::Rect {
                    frame: Frame::centered(0.0, 0.0, 2.0, len),
                    radius: 1.0,
                    color: ink,
                });
            }
        }
        self.close();
        if delta != 0 {
            let mut v = *value + step * delta as f32;
            if bounded {
                v = v.clamp(lo, hi);
            }
            let k = 10f32.powi(desc.decimals as i32);
            v = (v * k).round() / k;
            if v != *value {
                *value = v;
                out.changed = true;
            }
        }
        self.nodes[spin].a11y.value = Some(format!("{:.*}", desc.decimals as usize, *value));
        self.nodes[spin].a11y.range = Some(crate::a11y::A11yRange {
            min: bounded.then_some(lo),
            max: bounded.then_some(hi),
            now: *value,
        });
        out
    }

    /// A radio group: `Option#i` rows with a ring (filled amber when chosen)
    /// and a label.
    pub fn radio(
        &mut self,
        id: impl Into<Id>,
        options: &[&str],
        selected: &mut usize,
        desc: RadioDesc,
    ) -> Response {
        let t = self.theme;
        self.open(
            id,
            Element {
                direction: if desc.horizontal {
                    Direction::Row
                } else {
                    Direction::Column
                },
                gap: if desc.horizontal {
                    t.space.lg
                } else {
                    t.space.xs
                },
                ..Element::default()
            },
        );
        let group = self.parent();
        self.nodes[group].a11y.role = Some("radiogroup");
        let mut out = Response::default();
        for (k, label) in options.iter().enumerate() {
            let r = self.mark_control(
                crate::ui_idi("Option", k as u32),
                label,
                *selected == k,
                true,
            );
            if r.clicked && *selected != k {
                *selected = k;
                out.changed = true;
            }
            out.clicked |= r.clicked;
            out.focused |= r.focused;
            out.hovered |= r.hovered;
        }
        self.close();
        out
    }

    /// A checkbox with a label; a click flips it.
    pub fn checkbox(&mut self, id: impl Into<Id>, on: &mut bool, label: &str) -> Response {
        let mut r = self.mark_control(id.into(), label, *on, false);
        if r.clicked {
            *on = !*on;
            r.changed = true;
        }
        r
    }

    /// Checkboxes in a column (`Option#i`); `changed` when any flipped.
    pub fn checkbox_list(
        &mut self,
        id: impl Into<Id>,
        labels: &[&str],
        values: &mut [bool],
    ) -> Response {
        let t = self.theme;
        self.open(
            id,
            Element {
                direction: Direction::Column,
                gap: t.space.xs,
                ..Element::default()
            },
        );
        let group = self.parent();
        self.nodes[group].a11y.role = Some("group");
        let mut out = Response::default();
        for (k, label) in labels.iter().enumerate() {
            let mut v = values.get(k).copied().unwrap_or(false);
            let r = self.checkbox(crate::ui_idi("Option", k as u32), &mut v, label);
            if let Some(slot) = values.get_mut(k) {
                *slot = v;
            }
            out.changed |= r.changed;
            out.clicked |= r.clicked;
            out.focused |= r.focused;
        }
        self.close();
        out
    }

    /// A checkbox or radio row: the mark, then the label.
    fn mark_control(&mut self, id: Id, label: &str, on: bool, round: bool) -> Response {
        let t = self.theme;
        let p = t.palette;
        let box_ = self.sizes().control_sm - 2.0 * t.space.xs;
        let h = self.sizes().control_sm;
        let i = self.leaf(
            if round { "radio" } else { "checkbox" },
            id,
            Element {
                height: Size::Fixed(h),
                padding: Insets {
                    left: box_ + t.space.sm,
                    right: t.space.xs,
                    ..Insets::default()
                },
                interactive: true,
                radius: t.radius.control,
                ..Element::default()
            },
        );
        let r = self.nav_target(i, false);
        let mut c = self.text_content(label, t.type_ramp.body, p.fg);
        c.wrap = false;
        let n = &mut self.nodes[i];
        n.content = Content::Text(c);
        n.probe_text = label.to_owned();
        n.a11y.checked = Some(on);
        let frame = Frame {
            fy: 0.5,
            y: -box_ / 2.0,
            w: box_,
            h: box_,
            ..Frame::default()
        };
        let radius = if round { box_ } else { t.radius.control - 2.0 };
        if on {
            n.shapes.push(Shape::Rect {
                frame,
                radius,
                color: if r.pressed { p.accent_active } else { p.accent },
            });
            if round {
                let dot = box_ * 0.4;
                n.shapes.push(Shape::Rect {
                    frame: Frame {
                        fy: 0.5,
                        x: (box_ - dot) / 2.0,
                        y: -dot / 2.0,
                        w: dot,
                        h: dot,
                        ..Frame::default()
                    },
                    radius: dot,
                    color: p.accent_fg,
                });
            } else {
                // The check: a short and a long stroke.
                let s = box_ / 16.0;
                let w = 2.0f32.max(s * 2.0);
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fy: 0.5,
                        x: 3.5 * s,
                        y: 0.5 * s - w / 2.0,
                        w: 5.0 * s,
                        h: w,
                        ..Frame::default()
                    },
                    angle: FRAC_PI_4,
                    color: p.accent_fg,
                });
                n.shapes.push(Shape::Bar {
                    frame: Frame {
                        fy: 0.5,
                        x: 5.5 * s,
                        y: -1.5 * s - w / 2.0,
                        w: 9.0 * s,
                        h: w,
                        ..Frame::default()
                    },
                    angle: -FRAC_PI_4,
                    color: p.accent_fg,
                });
            }
        } else {
            n.shapes.push(Shape::Rect {
                frame,
                radius,
                color: p.border_strong,
            });
            let inset = 1.5;
            n.shapes.push(Shape::Rect {
                frame: Frame {
                    fy: 0.5,
                    x: inset,
                    y: -box_ / 2.0 + inset,
                    w: box_ - 2.0 * inset,
                    h: box_ - 2.0 * inset,
                    ..Frame::default()
                },
                radius: (radius - inset).max(0.0),
                color: if r.hovered {
                    p.surface_hover
                } else {
                    p.surface_raised
                },
            });
        }
        r
    }

    /// A chip: a capsule on `fill`; selected inverts to `fg` ink-on-light.
    pub fn chip(&mut self, id: impl Into<Id>, text: &str, desc: ChipDesc) -> Response {
        let t = self.theme;
        let p = t.palette;
        let h = self.sizes().control_sm;
        let i = self.leaf(
            "chip",
            id.into(),
            Element {
                height: Size::Fixed(h),
                padding: Insets::xy(t.space.md, 0.0),
                interactive: !desc.disabled,
                radius: t.radius.button,
                ..Element::default()
            },
        );
        self.nodes[i].disabled = desc.disabled;
        let r = if desc.disabled {
            Response::default()
        } else {
            self.nav_target(i, false)
        };
        let (bg, fg) = if desc.disabled {
            (p.fill, p.fg_subtle)
        } else if desc.selected {
            (p.fg, p.fg_inverse)
        } else if r.hovered || r.pressed {
            (p.fill_strong, p.fg)
        } else {
            (p.fill, p.fg)
        };
        let mut c = self.text_content(text, t.type_ramp.meta, fg);
        c.wrap = false;
        c.align = TextAlign::Center;
        let n = &mut self.nodes[i];
        n.el.background = Some(bg);
        n.content = Content::Text(c);
        n.probe_text = text.to_owned();
        n.a11y.selected = Some(desc.selected);
        r
    }

    /// A badge: a small status capsule in its tone (tinted fill, tone ink).
    ///
    /// Its ink reads [`READABLE`] on what it is drawn on. The tones and the
    /// accent are the palette's, made for its panels (where the packs' test
    /// holds them); on a ground on the other side of the scale from the
    /// palette's `surface` (a light pack's deep status colours on its dark
    /// stage) the ink moves towards its own end of the scale until it reads
    /// there (a deep green lightens on a dark screen; in the cream panel it
    /// stays deep). The neutral badge's ground is its own flat `fill`, so
    /// its `fgMuted` moves wherever it does not read on it, and where even
    /// white or black would not, the fill moves away from the ink instead
    /// (scifi's mid-blue fill darkens under its pale ink). og-taste §3.
    pub fn badge(&mut self, id: impl Into<Id>, text: &str, tone: Tone) {
        let t = self.theme;
        let p = t.palette;
        let (bg, fg) = match tone {
            Tone::Neutral => (p.fill, p.fg_muted),
            Tone::Accent => (p.accent_soft, p.accent_ink),
            Tone::Inverse => (p.fg, p.fg_inverse),
            // The status tones: the tone's own ink on its 16 % tint, mixed as
            // the platform's CSS mixes it (Srgba::tint_over, once the badge
            // knows what is behind it), so each pack's tones read 4.5:1.
            other => {
                let c = self.tone_dot(other);
                (c.fade(0.16), c)
            }
        };
        let i = self.leaf(
            "badge",
            id.into(),
            Element {
                height: Size::Fixed(t.type_ramp.xs.line_height + t.space.xs),
                padding: Insets::xy(t.space.sm, 0.0),
                background: Some(bg),
                radius: t.radius.button,
                ..Element::default()
            },
        );
        let under = self.behind(i);
        let (mut bg, ground) = match tone {
            // og-render composites the tint in linear light: tint_over
            // picks the alpha whose blend has the browser's luminance.
            Tone::Success | Tone::Warn | Tone::Danger | Tone::Info => {
                (fg.tint_over(under, 0.16), fg.fade(0.16).over(under))
            }
            _ => (bg, bg.over_linear(under)),
        };
        let dark = |c: Srgba| Srgba::WHITE.contrast(c) > Srgba::BLACK.contrast(c);
        let moves = match tone {
            Tone::Neutral => true,
            _ => dark(ground) != dark(p.surface),
        };
        // The ground is an estimate in bytes (the tint's sRGB mix), so a
        // moved ink aims a little over the bar.
        let aim = READABLE + 0.1;
        let fg = match fg.readable_on(ground, aim) {
            Some(ink) if moves => ink,
            None if moves => {
                if let Some(g) = ground.readable_on(fg, aim) {
                    bg = g;
                }
                fg
            }
            _ => fg,
        };
        let mut c = self.text_content(text, t.type_ramp.xs, fg);
        c.wrap = false;
        c.align = TextAlign::Center;
        let n = &mut self.nodes[i];
        n.el.background = Some(bg);
        n.content = Content::Text(c);
        n.probe_text = text.to_owned();
    }

    /// A hairline (`border`) across its parent; `vertical` stands it up.
    pub fn divider(&mut self, id: impl Into<Id>, vertical: bool) {
        let c = self.theme.palette.border;
        let px = 1.0 / self.viewport.dpr().max(1.0);
        let px = px.max(0.5);
        self.leaf(
            "divider",
            id.into(),
            Element {
                width: if vertical {
                    Size::Fixed(px)
                } else {
                    Size::grow()
                },
                height: if vertical {
                    Size::grow()
                } else {
                    Size::Fixed(px)
                },
                background: Some(c),
                ..Element::default()
            },
        );
    }

    /// Empty space: `size` along both axes (the zero value grows).
    pub fn spacer(&mut self, id: impl Into<Id>, size: Option<Size>) {
        let s = size.unwrap_or(Size::grow());
        let i = self.leaf(
            "spacer",
            id.into(),
            Element {
                width: s,
                height: s,
                ..Element::default()
            },
        );
        self.nodes[i].widget = Widget::Spacer;
    }

    /// A key and its value on one row (`fg-muted` key, `fg` value at the end).
    pub fn key_value(&mut self, id: impl Into<Id>, key: &str, value: &str, mono: bool) {
        let t = self.theme;
        let h = self.sizes().row;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                height: Size::Fit {
                    min: h * 0.75,
                    max: f32::INFINITY,
                },
                align: Align::SpaceBetween,
                cross: Cross::Center,
                gap: t.space.md,
                ..Element::default()
            },
        );
        self.label(
            "Key",
            key,
            crate::LabelDesc {
                color: Some(t.palette.fg_muted),
                nowrap: true,
                ..Default::default()
            },
        );
        self.label(
            "Value",
            value,
            crate::LabelDesc {
                nowrap: true,
                align: TextAlign::End,
                font: if mono { FontRole::Mono } else { FontRole::Body },
                ..Default::default()
            },
        );
        self.close();
    }

    /// A skeleton: the neutral fill standing in for content, with a sheen
    /// that passes every 1.6 s (still under reduced motion).
    pub fn skeleton(&mut self, id: impl Into<Id>, desc: SkeletonDesc) {
        let t = self.theme;
        let lines = desc.lines.max(1);
        let h = if desc.height > 0.0 {
            desc.height
        } else {
            t.type_ramp.body.size
        };
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or(Size::grow()),
                gap: t.space.sm,
                ..Element::default()
            },
        );
        let period = self.ticks_for(1600.0) as u64;
        let phase = (self.state.tick % period) as f32 / period as f32;
        for k in 0..lines {
            let last = lines > 1 && k == lines - 1;
            let i = self.leaf(
                "skeleton",
                Id::default(),
                Element {
                    width: if last {
                        Size::Percent(0.6)
                    } else {
                        Size::grow()
                    },
                    height: Size::Fixed(h),
                    background: Some(t.palette.fill),
                    radius: t.radius.control,
                    ..Element::default()
                },
            );
            if !t.reduced_motion {
                let band = 0.3;
                let x = -band + phase * (1.0 + band);
                let (a, b) = (x.max(0.0), (x + band).min(1.0));
                if b > a {
                    self.nodes[i].shapes.push(Shape::Rect {
                        frame: Frame::left(a, b),
                        radius: 0.0,
                        color: t.palette.sheen.fade(0.35),
                    });
                }
            }
        }
        self.close();
    }

    /// A loading spinner: eight dots around a circle, the bright one turning.
    pub fn spinner(&mut self, id: impl Into<Id>, size: f32) {
        let t = self.theme;
        let s = if size > 0.0 {
            size
        } else {
            self.sizes().control_sm - t.space.xs
        };
        let i = self.leaf(
            "spinner",
            id.into(),
            Element {
                width: Size::Fixed(s),
                height: Size::Fixed(s),
                ..Element::default()
            },
        );
        self.nodes[i].a11y.busy = true;
        self.nodes[i].a11y.name = Some("Loading".into());
        let period = self.ticks_for(t.durations_ms.slow * 2.0) as u64;
        let head = ((self.state.tick % period) * 8 / period) as i32;
        let dot = s / 5.0;
        for k in 0..8 {
            let ang = k as f32 * PI / 4.0 - FRAC_PI_2;
            let (dx, dy) = (
                (s / 2.0 - dot / 2.0) * cos(ang),
                (s / 2.0 - dot / 2.0) * sin(ang),
            );
            let age = (head - k).rem_euclid(8) as f32;
            self.nodes[i].shapes.push(Shape::Rect {
                frame: Frame::centered(dx, dy, dot, dot),
                radius: dot,
                color: t.palette.fg.fade(1.0 - age / 9.0),
            });
        }
    }

    /// An icon of the atlas (`set_icons`) by name; a missing name draws an
    /// empty slot and warns.
    pub fn icon(&mut self, id: impl Into<Id>, name: &str, desc: IconDesc) {
        let size = if desc.size > 0.0 { desc.size } else { 24.0 };
        let index = self.icons.index_of(name);
        if index.is_none() {
            self.warnings.push(format!(
                "icon '{name}' is not in the icon atlas; pass its name in the atlas's names"
            ));
        }
        let i = self.leaf(
            "icon",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                ..Element::default()
            },
        );
        let tint = desc.tint.unwrap_or(self.theme.palette.fg);
        match index.map(|k| (k, self.icons.glyph(k))) {
            Some((_, Some((font, ch)))) => self.nodes[i].shapes.push(Shape::Glyph {
                frame: Frame::FULL,
                font,
                ch,
                color: tint,
            }),
            Some((k, None)) if self.icons.texture.is_some() => {
                self.nodes[i].content = Content::Image {
                    texture: self.icons.texture,
                    uv: self.icons.uv(k),
                    tint,
                };
            }
            _ => {}
        }
        self.nodes[i].probe_text = name.to_owned();
    }

    /// Whether the icon atlas has an icon of this name.
    pub fn has_icon(&self, name: &str) -> bool {
        self.icons.is_drawn() && self.icons.index_of(name).is_some()
    }

    fn icon_shape(&self, name: &str, frame: Frame, color: Srgba) -> Option<Shape> {
        let k = self.icons.index_of(name)?;
        if let Some((font, ch)) = self.icons.glyph(k) {
            return Some(Shape::Glyph {
                frame,
                font,
                ch,
                color,
            });
        }
        Some(Shape::Image {
            frame,
            texture: self.icons.texture?,
            uv: self.icons.uv(k),
            color,
        })
    }

    // ── HUD ────────────────────────────────────────────────────────────────

    /// A meter (health, mana, xp): a capsule track and fill. A drop shows at
    /// once and leaves a pale trail that follows after a beat; a rise grows
    /// into place with a faint ghost of the target, by the tick.
    pub fn meter(&mut self, id: impl Into<Id>, value: f32, max: f32, desc: &MeterDesc) {
        let t = self.theme;
        let p = t.palette;
        let frac = if max > 0.0 && value.is_finite() {
            (value / max).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let h = if desc.height > 0.0 {
            desc.height
        } else if desc.label.is_empty() {
            t.space.md
        } else {
            t.type_ramp.caption.line_height + t.space.xxs
        };
        let i = self.leaf(
            "meter",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::grow()),
                height: Size::Fixed(h),
                background: Some(p.fill_strong),
                radius: t.radius.button,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let fresh = !self.has_anim(key);
        let mut a = self.anim(key);
        let now = self.state.tick;
        if fresh {
            a = Anim {
                a: frac,
                b: frac,
                t: now,
                h: 0,
            };
        }
        let hold = self.ticks_for(t.durations_ms.fast) as f32;
        let dur = self.ticks_for(t.durations_ms.slow) as f32;
        let shown = |a: &Anim| -> (f32, f32) {
            let el = (now - a.t) as f32;
            if a.b < a.a {
                // Dropped: the fill at the target, the trail from a towards b.
                let k = ease_out(((el - hold) / dur).clamp(0.0, 1.0));
                (a.b, a.a + (a.b - a.a) * k)
            } else {
                // Rose: the fill grows from a to b, a ghost at b.
                let k = ease_out((el / dur).clamp(0.0, 1.0));
                (a.a + (a.b - a.a) * k, a.b)
            }
        };
        if frac != a.b {
            // Start from what is on screen.
            let (main, ghost) = shown(&a);
            a.a = if frac < a.b { ghost.max(main) } else { main };
            a.b = frac;
            a.t = now;
        }
        let (main, ghost) = shown(&a);
        self.set_anim(key, a);
        let color = match desc.kind {
            MeterKind::Health if frac < 0.15 => p.danger,
            MeterKind::Health if frac < 0.3 => p.warn,
            MeterKind::Health => p.success,
            MeterKind::Mana => self.theme.hues.blue,
            MeterKind::Xp => self.theme.hues.purple,
            MeterKind::Accent => p.accent,
            MeterKind::Custom(c) => c,
        };
        // The label in two inks, each the one of fg and fgInverse that reads
        // on what it is drawn over: the fill left of its edge, the track
        // right of it (a light pack's deep status colours take the inverse).
        // Where neither reaches the bar (a mid blue or purple hue, a mid
        // accent), the better one moves on towards its end until it does,
        // else the other (white cannot read on cartoon's purple; black can).
        let behind = self.behind(i);
        let ink = |under: Srgba| {
            let under = under.over(behind);
            let (ink, other) = if p.fg_inverse.contrast(under) > p.fg.contrast(under) {
                (p.fg_inverse, p.fg)
            } else {
                (p.fg, p.fg_inverse)
            };
            ink.readable_on(under, READABLE)
                .or_else(|| other.readable_on(under, READABLE))
                .unwrap_or(ink)
        };
        let (fill_ink, track_ink) = (ink(color), ink(p.fill_strong));
        let n = &mut self.nodes[i];
        let seg = |from: f32, to: f32, c: Srgba| Shape::Rect {
            frame: Frame::left(from, to),
            radius: h / 2.0,
            color: c,
        };
        if ghost > main {
            let trail = if a.b < a.a || frac < ghost {
                p.fg.fade(0.55)
            } else {
                color.fade(0.4)
            };
            n.shapes.push(seg(0.0, ghost, trail));
        }
        if main > 0.0 {
            n.shapes.push(seg(0.0, main, color));
        }
        if desc.segments > 1 {
            for k in 1..desc.segments {
                let x = k as f32 / desc.segments as f32;
                n.shapes.push(Shape::Rect {
                    frame: Frame {
                        fx: x,
                        x: -1.0,
                        w: 2.0,
                        fh: 1.0,
                        ..Frame::default()
                    },
                    radius: 0.0,
                    color: p.bg.fade(0.55),
                });
            }
        }
        if !desc.label.is_empty() {
            for (clip, color) in [
                (Frame::left(0.0, main), fill_ink),
                (Frame::left(main, 1.0), track_ink),
            ] {
                if clip.fw > 0.0 {
                    n.shapes.push(Shape::Text {
                        frame: Frame::FULL,
                        text: desc.label.clone(),
                        step: t.type_ramp.caption,
                        color,
                        font: FontRole::Mono,
                        clip,
                    });
                }
            }
        }
        n.probe_text = desc.label.clone();
        n.a11y.value = Some(format!("{} of {}", value.round(), max.round()));
        n.a11y.range = Some(crate::a11y::A11yRange {
            min: Some(0.0),
            max: Some(max),
            now: value,
        });
    }

    /// A cooldown: a disc whose amber ring fills clockwise from the top as
    /// the cooldown runs out (`remaining` 1 → 0), with a label or icon in
    /// the middle; full and bright when ready. A press reports `clicked`
    /// (ready or not: the game decides).
    pub fn cooldown(&mut self, id: impl Into<Id>, remaining: f32, desc: &CooldownDesc) -> Response {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 56.0 }.max(self.min_target());
        let i = self.leaf(
            "cooldown",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: true,
                radius: size / 2.0,
                ..Element::default()
            },
        );
        let r = self.nav_target(i, false);
        let rem = if remaining.is_finite() {
            remaining.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let done = 1.0 - rem;
        let behind = self.behind(i);
        let track = p.fill_strong.over(p.surface_raised.over(behind));
        let fill = if rem <= 0.0 {
            p.accent
        } else {
            p.accent.mix(track, 0.25)
        };
        let inner = p.surface_raised.over(behind);
        let full = Frame::centered(0.0, 0.0, size, size);
        let mut shapes = vec![Shape::Rect {
            frame: full,
            radius: size,
            color: track,
        }];
        // The pie of `done` from the top, clockwise, as halves of the disc:
        // the right half, then another half that covers back (≤ 1/2) or extends (> 1/2).
        if done >= 1.0 {
            shapes.push(Shape::Rect {
                frame: full,
                radius: size,
                color: fill,
            });
        } else if done > 0.0 {
            let turn = 2.0 * PI * done;
            shapes.push(Shape::Half {
                frame: full,
                angle: 0.0,
                color: fill,
            });
            if done <= 0.5 {
                // Half a pixel larger, so no amber fringe shows at its edge.
                shapes.push(Shape::Half {
                    frame: Frame::centered(0.0, 0.0, size + 1.0, size + 1.0),
                    angle: turn,
                    color: track,
                });
            } else {
                shapes.push(Shape::Half {
                    frame: full,
                    angle: turn - PI,
                    color: fill,
                });
            }
        }
        let ring = t.space.xs.max(size / 14.0);
        let core = size - 2.0 * ring;
        shapes.push(Shape::Rect {
            frame: Frame::centered(0.0, 0.0, core, core),
            radius: core,
            color: if r.pressed {
                p.surface_hover.over(behind)
            } else {
                inner
            },
        });
        let ink = if rem <= 0.0 { p.fg } else { p.fg_muted };
        if !desc.icon.is_empty() {
            let s = core * 0.55;
            shapes.extend(self.icon_shape(&desc.icon, Frame::centered(0.0, 0.0, s, s), ink));
        } else if !desc.label.is_empty() {
            shapes.push(Shape::Text {
                frame: Frame::centered(0.0, 0.0, core, core),
                text: desc.label.clone(),
                step: t.type_ramp.lg,
                color: ink,
                font: FontRole::Mono,
                clip: Frame::FULL,
            });
        }
        let n = &mut self.nodes[i];
        n.shapes = shapes;
        n.probe_text = desc.label.clone();
        n.a11y.value = Some(if remaining <= 0.0 {
            "ready".into()
        } else {
            format!("{:.0}%", remaining.clamp(0.0, 1.0) * 100.0)
        });
        r
    }

    /// A zoomable picture (a map, an artwork): two fingers pinch it to zoom
    /// about their centre and pan it as they move (og.Pointers), the wheel
    /// zooms about the pointer, a drag pans; focused by keys or a pad, `+`
    /// (`=`) and `-` zoom it about its centre, `0` fits it again and the
    /// arrows pan it a tenth of its size. It stays on its picture and
    /// within its zoom limits; the zoom and the centre are UI state.
    /// Answers where it is.
    pub fn zoom_view(&mut self, id: impl Into<Id>, desc: &ZoomViewDesc) -> ZoomResponse {
        let i = self.leaf(
            "zoom",
            id.into(),
            Element {
                width: desc.width,
                height: desc.height,
                aspect: desc.aspect.max(0.0),
                // A scroller to the pointer, the wheel and two touches.
                scroll: true,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let (min, max) = match (desc.min_zoom, desc.max_zoom) {
            (a, b) if a > 0.0 && b >= a => (a, b),
            _ => (1.0, 4.0),
        };
        let s = self.zoom_state(key).unwrap_or(crate::scroll::ZoomState {
            zoom: min,
            center: og_core::glam::Vec2::splat(0.5),
            min,
            max,
        });
        let s = crate::scroll::ZoomState { min, max, ..s };
        self.set_zoom_state(key, s);
        // Keys and pads reach it; while focused its arrows (↑ ↓ too) are
        // its own, with + - and 0.
        let r = self.nav_target(i, true);
        self.nodes[i].multiline = true;
        if r.focused {
            let adjust = self.nav.adjust;
            self.zoom_keys(key, adjust);
        }
        let s = self.zoom_state(key).unwrap_or(s);
        let [u0, v0, u1, v1] = desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let half = 0.5 / s.zoom;
        let (fx0, fy0) = (s.center.x - half, s.center.y - half);
        let (fx1, fy1) = (s.center.x + half, s.center.y + half);
        let n = &mut self.nodes[i];
        n.content = Content::Image {
            texture: desc.texture,
            uv: [
                u0 + (u1 - u0) * fx0,
                v0 + (v1 - v0) * fy0,
                u0 + (u1 - u0) * fx1,
                v0 + (v1 - v0) * fy1,
            ],
            tint: desc.tint.unwrap_or(Srgba::WHITE),
        };
        // A picture named by its id (a map), its zoom as its value.
        n.a11y.role = Some("image");
        n.a11y.name = Some(n.id.clone());
        n.a11y.value = Some(format!("{:.0}%", s.zoom * 100.0));
        ZoomResponse {
            zoom: s.zoom,
            center: s.center,
        }
    }

    /// A mini-map frame: the map texture (or the stage colour) in a
    /// `border-strong` frame, markers as dots (the player amber and ringed),
    /// and an optional zone label under it.
    pub fn minimap(&mut self, id: impl Into<Id>, desc: &MinimapDesc) {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 160.0 };
        self.open(
            id,
            Element {
                direction: Direction::Column,
                gap: t.space.xs,
                cross: Cross::Center,
                ..Element::default()
            },
        );
        let i = self.leaf(
            "map",
            Id::from("Map"),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                padding: Insets::all(2.0),
                background: Some(p.border_strong),
                radius: t.radius.block,
                ..Element::default()
            },
        );
        let inner = Frame {
            x: 2.0,
            y: 2.0,
            fw: 1.0,
            fh: 1.0,
            w: -4.0,
            h: -4.0,
            ..Frame::default()
        };
        let mut shapes = vec![Shape::Rect {
            frame: inner,
            radius: t.radius.block - 2.0,
            color: p.stage_surface,
        }];
        if let Some(tex) = desc.texture {
            shapes.push(Shape::Image {
                frame: Frame {
                    x: 4.0,
                    y: 4.0,
                    fw: 1.0,
                    fh: 1.0,
                    w: -8.0,
                    h: -8.0,
                    ..Frame::default()
                },
                texture: tex,
                uv: desc.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
                color: Srgba::WHITE,
            });
        }
        let span = size - 8.0;
        for m in &desc.markers {
            let (x, y) = (
                (m.x.clamp(0.0, 1.0) - 0.5) * span,
                (m.y.clamp(0.0, 1.0) - 0.5) * span,
            );
            if m.player {
                let d = t.space.md;
                shapes.push(Shape::Rect {
                    frame: Frame::centered(x, y, d + 4.0, d + 4.0),
                    radius: d,
                    color: p.stage,
                });
                shapes.push(Shape::Rect {
                    frame: Frame::centered(x, y, d, d),
                    radius: d,
                    color: p.accent,
                });
            } else {
                let d = t.space.sm;
                let c = if m.tone == Tone::Neutral {
                    p.stage_fg
                } else {
                    self.tone_dot(m.tone)
                };
                shapes.push(Shape::Rect {
                    frame: Frame::centered(x, y, d, d),
                    radius: d,
                    color: c,
                });
            }
        }
        shapes.push(Shape::Text {
            frame: Frame {
                fx: 0.5,
                x: -10.0,
                y: 6.0,
                w: 20.0,
                h: 14.0,
                ..Frame::default()
            },
            text: "N".into(),
            step: t.type_ramp.xs,
            color: p.stage_muted,
            font: FontRole::Body,
            clip: Frame::FULL,
        });
        self.nodes[i].shapes = shapes;
        if !desc.label.is_empty() {
            self.label(
                "Zone",
                &desc.label,
                crate::LabelDesc {
                    step: Some(t.type_ramp.meta),
                    color: Some(p.fg_muted),
                    nowrap: true,
                    ..Default::default()
                },
            );
        }
        self.close();
    }

    /// A dialogue box: a portrait slot, the speaker in amber ink, and the
    /// line typed out by the tick at `chars_per_second`. A press (or
    /// Activate) while typing shows the whole line; once shown, it answers
    /// `advanced`. A new `text` starts typing again.
    pub fn dialogue(&mut self, id: impl Into<Id>, desc: &DialogueDesc) -> DialogueResponse {
        let t = self.theme;
        let p = t.palette;
        let i = self.push(
            "dialogue",
            id.into(),
            Element {
                direction: Direction::Row,
                width: Size::grow(),
                padding: Insets::all(t.space.lg),
                gap: t.space.lg,
                cross: Cross::Start,
                background: Some(p.surface_raised),
                radius: t.radius.panel,
                interactive: true,
                ..Element::default()
            },
        );
        self.stack.push(i);
        let key = self.nodes[i].key;
        // A dialogue that appears takes the focus (the topmost owns it: the
        // og-ui skill §4, README §Focus navigation), so Enter, Space and the
        // pad's A reach it at once; under an open dialog it waits.
        if self.state.scope == 0 && !self.state.hits.iter().any(|h| h.key == key) {
            self.request_focus = key;
        }
        let r = self.nav_target(i, false);
        let h = hash_str(&desc.text);
        let mut a = self.anim(key);
        let now = self.state.tick;
        if a.h != h || a.t == 0 {
            a = Anim {
                a: 0.0,
                b: 0.0,
                t: now,
                h,
            };
        }
        let cps = if desc.chars_per_second > 0.0 {
            desc.chars_per_second
        } else {
            40.0
        };
        let total = desc.text.chars().count() as u32;
        let typed = if a.a >= 1.0 {
            total
        } else {
            ((((now - a.t) as f32) * cps / self.ticks_per_second as f32).floor() as u32).min(total)
        };
        let mut out = DialogueResponse {
            shown: typed,
            done: typed >= total,
            advanced: false,
        };
        if r.clicked {
            if out.done {
                out.advanced = true;
            } else {
                a.a = 1.0;
                out.shown = total;
                out.done = true;
            }
        }
        self.set_anim(key, a);
        let portrait = 72.0;
        let portrait_slot = |ui: &mut Ui| {
            let s = ui.leaf(
                "portrait",
                Id::from("Portrait"),
                Element {
                    width: Size::Fixed(portrait),
                    height: Size::Fixed(portrait),
                    background: Some(p.fill),
                    radius: t.radius.card,
                    ..Element::default()
                },
            );
            let icon = if desc.portrait_icon.is_empty() {
                None
            } else {
                ui.icon_shape(
                    &desc.portrait_icon,
                    Frame::centered(0.0, 0.0, portrait * 0.7, portrait * 0.7),
                    p.fg,
                )
            };
            match (desc.portrait, icon) {
                (Some((tex, uv)), _) => {
                    ui.nodes[s].content = Content::Image {
                        texture: Some(tex),
                        uv,
                        tint: Srgba::WHITE,
                    };
                }
                (None, Some(shape)) => ui.nodes[s].shapes.push(shape),
                (None, None) => {
                    let initial: String = desc.speaker.chars().take(1).collect();
                    ui.nodes[s].shapes.push(Shape::Text {
                        frame: Frame::FULL,
                        text: initial,
                        step: t.type_ramp.display_sm,
                        color: p.fg_muted,
                        font: FontRole::Display,
                        clip: Frame::FULL,
                    });
                }
            }
        };
        if !desc.portrait_right {
            portrait_slot(self);
        }
        self.open(
            "Body",
            Element {
                direction: Direction::Column,
                width: Size::grow(),
                height: Size::Fit {
                    min: portrait,
                    max: f32::INFINITY,
                },
                gap: t.space.xs,
                ..Element::default()
            },
        );
        if !desc.speaker.is_empty() {
            self.label(
                "Speaker",
                &desc.speaker,
                crate::LabelDesc {
                    step: Some(t.type_ramp.meta),
                    color: Some(p.accent_ink),
                    nowrap: true,
                    ..Default::default()
                },
            );
        }
        let shown: String = desc.text.chars().take(out.shown as usize).collect();
        self.label(
            "Line",
            &shown,
            crate::LabelDesc {
                step: Some(t.type_ramp.reading),
                width: Size::grow(),
                ..Default::default()
            },
        );
        self.close();
        if desc.portrait_right {
            portrait_slot(self);
        }
        if out.done {
            // The continue mark: a small chevron that bobs.
            let bob = if t.reduced_motion {
                0.0
            } else {
                ((now / self.ticks_for(t.durations_ms.slow) as u64) % 2) as f32 * 2.0
            };
            for side in [-1.0f32, 1.0] {
                self.nodes[i].shapes.push(Shape::Bar {
                    frame: Frame {
                        fx: 1.0,
                        fy: 1.0,
                        x: -t.space.lg - 6.0 + side * 2.5 - 4.0,
                        y: -t.space.md - 2.0 + bob,
                        w: 8.0,
                        h: 2.0,
                        ..Frame::default()
                    },
                    angle: side * FRAC_PI_4,
                    color: p.fg_muted,
                });
            }
        }
        self.nodes[i].probe_text = shown;
        self.close();
        out
    }

    // ── touch controls ─────────────────────────────────────────────────────

    /// A virtual joystick: a base disc and a knob that follows a press that
    /// started on it, clamped to the base. Answers the stick's -1..1 vector.
    /// It keeps the finger that grabbed it (any of og.Pointers), so another
    /// finger can hold a button at the same time.
    pub fn joystick(&mut self, id: impl Into<Id>, desc: JoystickDesc) -> JoystickResponse {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 128.0 }.max(3.0 * self.min_target());
        let i = self.leaf(
            "joystick",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: true,
                radius: size / 2.0,
                background: Some(p.material_thick),
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let r = self.respond(key);
        let mut out = JoystickResponse::default();
        let radius = size / 2.0;
        let (mut kx, mut ky) = (0.0, 0.0);
        // The primary pointer holding it, else a further touch that grabbed it.
        let finger = if self.state.active == key && (self.input.down || self.input.pressed) {
            Some(self.input.pointer)
        } else {
            self.touch(key, true).filter(|t| t.1).map(|t| t.0)
        };
        if let Some(finger) = finger
            && let Some(h) = self.hit(key)
        {
            let (cx, cy) = h.rect.center();
            let (mut dx, mut dy) = ((finger.x - cx) / radius, (finger.y - cy) / radius);
            let len = (dx * dx + dy * dy).sqrt();
            if len > 1.0 {
                dx /= len;
                dy /= len;
            }
            kx = dx;
            ky = dy;
            let dz = if desc.dead_zone > 0.0 {
                desc.dead_zone
            } else {
                0.15
            };
            if len.min(1.0) > dz {
                out.x = dx;
                out.y = dy;
            }
            out.active = true;
        }
        let knob = size * 0.42;
        let travel = radius - knob / 2.0;
        let n = &mut self.nodes[i];
        n.shapes.push(Shape::Rect {
            frame: Frame::centered(0.0, 0.0, size - 2.0 * t.space.lg, size - 2.0 * t.space.lg),
            radius: size,
            color: p.fill,
        });
        n.shapes.push(Shape::Rect {
            frame: Frame::centered(kx * travel, ky * travel, knob, knob),
            radius: knob,
            color: if r.pressed {
                p.fg
            } else {
                p.thumb.mix(p.fg, 0.35)
            },
        });
        out
    }

    /// A d-pad: four arms (`Up`, `Down`, `Left`, `Right`), each at least the
    /// touch target, answering which are held.
    pub fn dpad(&mut self, id: impl Into<Id>, desc: DpadDesc) -> DpadResponse {
        let t = self.theme;
        let p = t.palette;
        let arm = (if desc.size > 0.0 { desc.size } else { 132.0 } / 3.0).max(self.min_target());
        // Nine cells in a wrapping row three wide: the arms are the pad's
        // children. The probe names the row `dpad` (a touch control).
        let pad = self.push(
            "dpad",
            id.into(),
            Element {
                direction: Direction::Row,
                wrap: true,
                width: Size::Fixed(arm * 3.0),
                height: Size::Fixed(arm * 3.0),
                ..Element::default()
            },
        );
        self.stack.push(pad);
        let mut out = DpadResponse::default();
        let cells: [Option<(&str, f32)>; 9] = [
            None,
            Some(("Up", -FRAC_PI_2)),
            None,
            Some(("Left", PI)),
            None,
            Some(("Right", 0.0)),
            None,
            Some(("Down", FRAC_PI_2)),
            None,
        ];
        for (k, cell) in cells.iter().enumerate() {
            match cell {
                None => {
                    let centre = k == 4;
                    let i = self.leaf(
                        "spacer",
                        Id::default(),
                        Element {
                            width: Size::Fixed(arm),
                            height: Size::Fixed(arm),
                            background: centre.then_some(p.material_thick),
                            ..Element::default()
                        },
                    );
                    self.nodes[i].widget = Widget::Spacer;
                }
                Some((name, angle)) => {
                    let i = self.leaf(
                        "button",
                        Id::from(*name),
                        Element {
                            width: Size::Fixed(arm),
                            height: Size::Fixed(arm),
                            interactive: true,
                            radius: t.radius.block,
                            background: Some(p.material_thick),
                            ..Element::default()
                        },
                    );
                    let key = self.nodes[i].key;
                    let r = self.respond(key);
                    let held = r.pressed || self.touch(key, false).is_some_and(|t| t.1);
                    match *name {
                        "Up" => out.up = held,
                        "Down" => out.down = held,
                        "Left" => out.left = held,
                        _ => out.right = held,
                    }
                    let ink = if held { p.accent } else { p.fg };
                    let n = &mut self.nodes[i];
                    if held {
                        n.el.background = Some(p.fill_strong.over(p.material_thick.over(p.bg)));
                    }
                    n.probe_text = (*name).to_owned();
                    // A chevron pointing `angle`: two strokes meeting at its tip.
                    let len = arm * 0.24;
                    let (ca, sa) = (cos(*angle), sin(*angle));
                    let tip = (ca * len * 0.35, sa * len * 0.35);
                    for side in [-1.0f32, 1.0] {
                        let back = *angle + side * 3.0 * FRAC_PI_4;
                        let (ox, oy) =
                            (tip.0 + cos(back) * len * 0.5, tip.1 + sin(back) * len * 0.5);
                        n.shapes.push(Shape::Bar {
                            frame: Frame::centered(ox, oy, len, 3.0),
                            angle: back,
                            color: ink,
                        });
                    }
                }
            }
        }
        self.close();
        out
    }

    /// A round action button (jump, fire): at least the touch target;
    /// `pressed` while held, `clicked` on a tap.
    pub fn action_button(
        &mut self,
        id: impl Into<Id>,
        text: &str,
        desc: &ActionButtonDesc,
    ) -> Response {
        let t = self.theme;
        let p = t.palette;
        let size = if desc.size > 0.0 { desc.size } else { 64.0 }.max(self.min_target());
        let i = self.leaf(
            "action",
            id.into(),
            Element {
                width: Size::Fixed(size),
                height: Size::Fixed(size),
                interactive: true,
                radius: size / 2.0,
                ..Element::default()
            },
        );
        let key = self.nodes[i].key;
        let mut r = self.respond(key);
        // A further touch on the button: held while over it, a click when it
        // lifts over it (with the primary pointer busy on a stick).
        if let Some((_, held)) = self.touch(key, false) {
            r.pressed |= held;
            r.clicked |= !held;
        }
        let (bg, ink) = if desc.primary {
            (
                if r.pressed { p.accent_active } else { p.accent },
                p.accent_fg,
            )
        } else {
            (
                if r.pressed {
                    p.fill_strong.over(p.material_thick.over(p.bg))
                } else {
                    p.material_thick
                },
                p.fg,
            )
        };
        let icon = if desc.icon.is_empty() {
            None
        } else {
            let s = size * 0.45;
            self.icon_shape(&desc.icon, Frame::centered(0.0, 0.0, s, s), ink)
        };
        let n = &mut self.nodes[i];
        n.el.background = Some(bg);
        n.widget = Widget::Button {
            pressed: r.pressed,
            press: if r.pressed { 1.0 } else { 0.0 },
        };
        n.probe_text = text.to_owned();
        match icon {
            Some(s) => n.shapes.push(s),
            None => n.shapes.push(Shape::Text {
                frame: Frame::FULL,
                text: text.to_owned(),
                step: t.type_ramp.sm,
                color: ink,
                font: FontRole::Body,
                clip: Frame::FULL,
            }),
        }
        r
    }
}

fn cos(a: f32) -> f32 {
    a.cos()
}

fn sin(a: f32) -> f32 {
    a.sin()
}
