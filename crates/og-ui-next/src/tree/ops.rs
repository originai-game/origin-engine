//! The TS renderer's ops applied to the arena (CONVENTIONS.md §Ops): a
//! binary batch read in place, each op patching the element it names. An
//! op node is an element of the tree like a document's, so layout, text,
//! icons and paint serve it unchanged; its record comes from its class and
//! style through the resolver, once per distinct pair.

use super::*;
use crate::a11y;
use rustc_hash::FxHashMap;

/// The first bytes of an op batch, which no store command batch starts
/// with (the wire codec's first byte is a value tag, 0 to 6).
pub const MAGIC: &[u8; 4] = b"OGOP";
/// The first bytes of the reply: the events since the last batch.
pub const EVENTS: &[u8; 4] = b"OGEV";
/// The first bytes of the reply to a batch an op refused, then the reason.
pub const REFUSED: &[u8; 4] = b"OGER";

/// What an op node may listen to, by its code in `listen`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Press = 0,
    Tap,
    PointerDown,
    PointerMove,
    PointerUp,
    Wheel,
    Focus,
    Blur,
    Key,
    Change,
    /// An input action its `actions` prop lists (`onAction`), by name in
    /// the event's `key`.
    Action,
    /// A finger held still on it for 0.5 s (`onLongPress`).
    LongPress,
    /// A held press on a node with a `hold` duration: its progress, 0 to
    /// 1, in `value` each frame; "0" when it lets go early (`onHold`).
    Hold,
    /// A `draggable` node was picked up: its data in `key`, its box in
    /// `value` (`onDragStart`).
    DragStart,
    /// A carried node moved: its data in `key`, the `dropTarget` under it
    /// (or focused) in `value`, "" over none (`onDragMove`).
    DragMove,
    /// A carried node was put down: its data in `key`, the `dropTarget` it
    /// went to in `value`, "" when cancelled (`onDragEnd`).
    DragEnd,
    /// A carried node was dropped on this `dropTarget`: the node's data in
    /// `key`, this target's box in `value` (`onDrop`).
    Drop,
    /// Two fingers that went down on it pinch: `key` is `start`, `move`
    /// or `end`, `value` the scale from where they began, x and y their
    /// midpoint (`onPinch`).
    Pinch,
    /// Two fingers that went down on it turn: as `Pinch`, `value` the
    /// turn in radians, clockwise on screen (`onRotate`).
    Rotate,
    /// A pointer that went down on it was let go moving fast: `key` its
    /// direction (`left`, `right`, `up`, `down`), `value` its velocity
    /// `"vx vy"` in px/s (`onSwipe`).
    Swipe,
    /// A context menu was asked for on it: the secondary button went down
    /// over it (`x`, `y` the pointer), or Shift+F10 or the Menu key was
    /// pressed while the focus was in it (`value` `"key"`, `x`, `y` its
    /// box's bottom start) (`onContextMenu`).
    ContextMenu,
    /// Its scroll moved (a `Scroll`, or overflow that scrolls), once a
    /// frame: `x`, `y` the offset, `value` `"viewportWidth viewportHeight
    /// contentWidth contentHeight"` (`onScroll`).
    Scroll,
}

impl Event {
    const ALL: [Event; 22] = [
        Event::Press,
        Event::Tap,
        Event::PointerDown,
        Event::PointerMove,
        Event::PointerUp,
        Event::Wheel,
        Event::Focus,
        Event::Blur,
        Event::Key,
        Event::Change,
        Event::Action,
        Event::LongPress,
        Event::Hold,
        Event::DragStart,
        Event::DragMove,
        Event::DragEnd,
        Event::Drop,
        Event::Pinch,
        Event::Rotate,
        Event::Swipe,
        Event::ContextMenu,
        Event::Scroll,
    ];

    pub fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// An event an op node fired, for the renderer to dispatch by id.
#[derive(Clone, Debug, PartialEq)]
pub struct Fired {
    pub id: u32,
    pub event: Event,
    /// The pointer on the UI's surface.
    pub x: f32,
    pub y: f32,
    /// A key's name (`key`) and a control's new value (`change`).
    pub key: Box<str>,
    pub value: Box<str>,
}

impl Fired {
    pub(super) fn at(id: u32, event: Event, x: f32, y: f32) -> Fired {
        Fired {
            id,
            event,
            x,
            y,
            key: Box::default(),
            value: Box::default(),
        }
    }
}

/// A class string and inline style to their record under the given
/// [`crate::classes::state`] bits: 0, or `STARTING` and `EXIT` for the
/// records motion enters from and leaves to.
/// `data` is the node's data context ([`Classes::data_id`]): its own
/// `data-*` names and its nearest groups' as `group/name`, by spaces.
pub type Resolve = Box<dyn Fn(&str, &str, u16, &str) -> Record>;

use crate::classes::state::{
    ARIA_PRESSED, BUSY, CHECKED, DISABLED, EXIT, EXPANDED, FOCUS_VISIBLE, FOCUS_WITHIN, FOCUSED,
    HOVERED, INDETERMINATE, INVALID, PRESSED, SELECTED, STARTING,
};

mod cache;
mod inspect;
mod lifecycle;
pub use cache::Classes;
pub(super) use cache::GROUP_STATES;

/// A batch read in place.
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.b.len());
        let end = end.ok_or("og.ui ops: the batch ends inside an op")?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }
    fn str(&mut self) -> Result<&'a str, String> {
        let n = self.u32()? as usize;
        std::str::from_utf8(self.take(n)?).map_err(|_| "og.ui ops: a string is not UTF-8".into())
    }
}

impl Tree {
    fn apply_ops(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut r = Reader { b: bytes, at: 0 };
        let count = r.u32()?;
        for _ in 0..count {
            let code = r.u8()?;
            match code {
                // create
                1 => {
                    let id = r.u32()?;
                    let tag = r.u8()?;
                    let kind = match tag {
                        1 => Kind::Text,
                        2 => Kind::Image,
                        5 => Kind::Icon,
                        0..=7 => Kind::Box,
                        t => return Err(format!("og.ui ops: unknown tag {t}")),
                    };
                    // An `Input` shows its text in a text leaf of its own.
                    let e = self.create(id, kind, tag == 4)?;
                    self.input.per[e as usize].scroller = tag == 3;
                    if tag == 6 {
                        self.els[e as usize].line = Some(Box::default());
                    }
                    if tag == 7 {
                        self.els[e as usize].particles = Some(Box::default());
                    }
                    self.input.per[e as usize].input = tag == 4;
                    self.sem
                        .create(self.els[e as usize].node, a11y::Kind::of(tag));
                    if tag == 4 {
                        self.field_class(e);
                    }
                }
                // createText
                2 => {
                    let id = r.u32()?;
                    let text = r.str()?;
                    let e = self.create(id, Kind::Label, true)?;
                    self.set_text(e, text);
                    self.sem.create(self.els[e as usize].node, a11y::Kind::Text);
                }
                // setText
                3 => {
                    let e = self.op(r.u32()?)?;
                    let text = r.str()?;
                    self.set_text(e, text);
                }
                // setClass, setStyle
                4 | 5 => {
                    let e = self.op(r.u32()?)?;
                    let text = r.str()?;
                    // An `Input`'s class keeps its default field slot.
                    if code == 4 && self.input.per[e as usize].input {
                        self.input.per[e as usize].own_class = text.into();
                        self.field_class(e);
                        continue;
                    }
                    let old = self.els[e as usize].class;
                    let new = self.classes.with(old, code == 4, text);
                    self.swap_class(e, new);
                    if code == 4 {
                        let slot = text.split_whitespace().find_map(|c| c.strip_prefix('@'));
                        let per = &mut self.input.per[e as usize];
                        if *per.slot != *slot.unwrap_or("") {
                            per.slot = slot.unwrap_or("").into();
                        }
                    }
                    self.data_dirty = true;
                    self.restyle(e);
                }
                // setProp
                6 => {
                    let e = self.op(r.u32()?)?;
                    let (name, value) = (r.str()?, r.str()?);
                    let node = self.els[e as usize].node;
                    let was = self.sem.get(node).range[2];
                    self.sem.prop(node, name, value);
                    // A slider moved to a new value sounds its `step`.
                    let s = self.sem.get(node);
                    if name == "valueNow"
                        && s.role == Some(Some(accesskit::Role::Slider))
                        && was.is_some()
                        && s.range[2] != was
                    {
                        self.sound(e, "step");
                    }
                    match name {
                        "disabled" => {
                            let on = !matches!(value, "" | "false" | "0");
                            let was = self.els[e as usize].disabled;
                            self.els[e as usize].disabled = on;
                            self.set_state(e, DISABLED, on);
                            // A control drawn disabled that becomes ready
                            // calls for attention once (`Attention.cue`).
                            if was && !on && self.els[e as usize].entering.is_none() {
                                let cue = self.classes.get("@Attention.cue", "");
                                let lp = self.classes.record(cue, 0, 0).motion.animate;
                                if let Some(lp) = lp {
                                    let el = &self.els[e as usize];
                                    self.motion.cue(el.node, lp, &el.record.skin);
                                    self.paint = true;
                                }
                            }
                        }
                        "tabIndex" => {
                            self.input.per[e as usize].tab =
                                value.trim().parse().unwrap_or(input::NO_TAB);
                        }
                        "modal" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].modal = on;
                            self.input.modal_els.retain(|&m| m != e);
                            if on {
                                self.input.modal_els.push(e);
                            }
                        }
                        "actions" => self.input.per[e as usize].actions = value.into(),
                        "autofocus" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].autofocus = on;
                            self.input.autofocus.retain(|&a| a != e);
                            if on {
                                self.input.autofocus.push(e);
                            }
                        }
                        "hold" => {
                            self.input.per[e as usize].hold = value.trim().parse().unwrap_or(0.0);
                        }
                        "touchStick" | "touchPad" | "touchButton" => {
                            let kind = match name {
                                "touchStick" => og_input::ZoneKind::Stick,
                                "touchPad" => og_input::ZoneKind::Pad,
                                _ => og_input::ZoneKind::Button,
                            };
                            let action = value.trim();
                            let per = &mut self.input.per[e as usize];
                            if !action.is_empty() {
                                per.zone = Some((kind, action.into()));
                            } else if per.zone.as_ref().is_some_and(|z| z.0 == kind) {
                                per.zone = None;
                            }
                            let zone = per.zone.is_some();
                            self.input.zone_els.retain(|&z| z != e);
                            if zone {
                                self.input.zone_els.push(e);
                            }
                            // A touch button squashes by the pack's press, or no longer.
                            self.restyle(e);
                        }
                        "value" | "placeholder" if self.input.per[e as usize].input => {
                            let p = &mut self.input.per[e as usize];
                            if name == "placeholder" {
                                p.placeholder = value.into();
                            } else if p.value != value {
                                // Set from outside: the caret goes to the end.
                                p.value.clear();
                                p.value.push_str(value);
                                p.composing.clear();
                                p.caret = p.value.len();
                                p.anchor = p.caret;
                                // Another text: what undo knew is gone.
                                p.undo.clear();
                                p.redo.clear();
                                p.last_edit = Default::default();
                            }
                            self.show_value(e);
                        }
                        "type" if self.input.per[e as usize].input => {
                            let v = value.trim();
                            let p = &mut self.input.per[e as usize];
                            p.input_type = v.into();
                            if v == "password" {
                                p.secret = true;
                            }
                            self.show_value(e);
                        }
                        "inputMode" | "inputmode" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].input_mode = value.trim().into();
                        }
                        "autoComplete" | "autocomplete" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].autocomplete = value.trim().into();
                        }
                        "secret" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].secret =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.show_value(e);
                        }
                        "multiline" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].multiline =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.field_class(e);
                            self.inherit(e);
                        }
                        // `rows`: a multi-line field that many lines tall.
                        "rows" if self.input.per[e as usize].input => {
                            let rows = value.trim().parse::<u16>().unwrap_or(0);
                            let p = &mut self.input.per[e as usize];
                            p.rows = rows;
                            p.multiline |= rows > 1;
                            self.field_class(e);
                            self.inherit(e);
                        }
                        "maxLength" | "maxlength" if self.input.per[e as usize].input => {
                            self.input.per[e as usize].max_length =
                                value.trim().parse::<usize>().unwrap_or(0);
                        }
                        "readOnly" | "readonly" if self.input.per[e as usize].input => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            self.input.per[e as usize].read_only = on;
                            // `data-readonly:` classes draw it.
                            self.mark(e, "readonly", on);
                        }
                        "focusGroup" => {
                            self.input.per[e as usize].group = match value.trim() {
                                "wrap" => Some(true),
                                "stop" | "true" => Some(false),
                                _ => None,
                            };
                        }
                        "focusMemory" => {
                            self.input.per[e as usize].memory =
                                !matches!(value, "" | "false" | "0" | "null");
                        }
                        "focusRoving" => {
                            self.input.per[e as usize].roving =
                                !matches!(value, "" | "false" | "0" | "null");
                        }
                        "draggable" => self.input.per[e as usize].drag = value.trim().into(),
                        "dropTarget" => self.input.per[e as usize].drop = value.trim().into(),
                        "pinned" => {
                            let on = !matches!(value, "" | "false" | "0" | "null");
                            let node = self.els[e as usize].node;
                            self.input.per[e as usize].pinned = on;
                            self.input.pinned.retain(|&n| n != node);
                            if on {
                                self.input.pinned.push(node);
                            }
                            self.paint = true;
                        }
                        "thumb" => {
                            self.input.per[e as usize].thumb = value.trim().into();
                            if !self.input.thumbs.contains(&e) {
                                self.input.thumbs.push(e);
                            }
                            self.inherit(e);
                        }
                        "knob" => {
                            self.input.per[e as usize].rides_knob =
                                !matches!(value, "" | "false" | "0" | "null");
                            self.paint = true;
                        }
                        "deadZone" => {
                            self.input.per[e as usize].dead_zone =
                                value.trim().parse::<f32>().unwrap_or(0.0).clamp(0.0, 0.95);
                        }
                        "glyph" => {
                            self.input.per[e as usize].glyph = value.into();
                            self.input.glyphs_wanted = true;
                            // The glyph is the node's text: it lays out in
                            // a text leaf of its own, as an `Input`'s does.
                            if self.els[e as usize].leaf.is_none() {
                                let node = self.els[e as usize].node;
                                self.els[e as usize].leaf = Some(self.arena.text(node, "", 16.0));
                                self.inherit(e);
                            }
                        }
                        // A `<Line>`'s points, numbers split by commas or
                        // spaces, read into its own buffer; `viewBox` the
                        // points' space (`w h`, or SVG's `x y w h`).
                        "points" | "closed" | "viewBox" | "fit"
                            if self.els[e as usize].line.is_some() =>
                        {
                            let line = self.els[e as usize].line.as_mut().expect("a line");
                            let numbers = |v: &str| -> Vec<f32> {
                                v.split([',', ' '])
                                    .filter(|t| !t.is_empty())
                                    .map(|t| t.parse::<f32>().unwrap_or(0.0))
                                    .collect()
                            };
                            match name {
                                "closed" => line.closed = !matches!(value, "" | "false" | "0"),
                                "fit" => line.contain = value == "contain",
                                "viewBox" => {
                                    line.view = match numbers(value)[..] {
                                        [w, h] => Some([0.0, 0.0, w, h]),
                                        [x, y, w, h] => Some([x, y, w, h]),
                                        _ => None,
                                    }
                                }
                                _ => {
                                    line.points = numbers(value);
                                    line.points.truncate(line.points.len() & !1);
                                }
                            }
                            self.paint = true;
                        }
                        // A `<Particles>`' emitter, seed, delay and restart
                        // key (`particles`).
                        "emitter" | "seed" | "delay" | "restart"
                            if self.els[e as usize].particles.is_some() =>
                        {
                            self.set_particles(e, name, value);
                        }
                        // `sound`: the pack event the node sounds when it
                        // first paints (its slot's, else the pack's).
                        "sound" => {
                            self.input.per[e as usize].cue = value.into();
                            self.paint = true;
                        }
                        // `exitSound`: the pack event the node sounds when
                        // removed/unmounted (e.g. `"close"` on overlays).
                        "exitSound" => {
                            self.input.per[e as usize].exit_cue = value.into();
                        }
                        // `haptic`: preset override played with this node's
                        // non-hover interactions and cues (`"none"` silences).
                        "haptic" => {
                            self.input.per[e as usize].haptic = value.into();
                        }
                        // `anchor` an entity's number or a world point
                        // `x,y,z` (empty: back in flow): the host places
                        // the node (§World); any other text is a UI
                        // node's `name`, which the tree places it on.
                        "anchor" => {
                            let v = value.trim();
                            let node_name = !v.is_empty()
                                && v.split([',', ' '])
                                    .any(|t| !t.is_empty() && t.parse::<f64>().is_err());
                            let n: Vec<f32> = value
                                .split([',', ' '])
                                .filter(|t| !t.is_empty())
                                .map(|t| t.parse().unwrap_or(0.0))
                                .collect();
                            let target = match n[..] {
                                _ if node_name => {
                                    Some(super::AnchorTarget::Node(self.name_index(v)))
                                }
                                [] => None,
                                [x, y, z] => Some(super::AnchorTarget::Point([x, y, z])),
                                _ => value
                                    .trim()
                                    .parse::<f64>()
                                    .ok()
                                    .map(|b| super::AnchorTarget::Entity(b as u64)),
                            };
                            let node = self.els[e as usize].node;
                            let a = self.els[e as usize]
                                .anchor
                                .get_or_insert_with(|| Box::new(super::Anchor::new()));
                            let was = a.target.is_some();
                            a.target = target;
                            match target {
                                Some(_) if was => {}
                                // Off the screen until the host places it.
                                Some(_) => {
                                    (a.shown, a.ticked, a.seen) = (false, false, None);
                                    self.anchored.push(e);
                                    self.sem.place(node, None);
                                    self.reapply(e);
                                }
                                None if was => {
                                    self.anchored.retain(|&a| a != e);
                                    self.sem.unplace(node);
                                    for name in [
                                        "clamped",
                                        "edge-top",
                                        "edge-right",
                                        "edge-bottom",
                                        "edge-left",
                                        "occluded",
                                        "lod-near",
                                        "lod-mid",
                                        "lod-far",
                                    ] {
                                        self.mark(e, name, false);
                                    }
                                    self.reapply(e);
                                }
                                None => {}
                            }
                            self.paint = true;
                        }
                        // How the host places it (§World): `lift` a world
                        // offset, `occlusion` none, fade or hide behind
                        // geometry, `declutter` none, stack or hide over
                        // another, `priority` which keeps its place,
                        // `clamp` edge pins it inside the screen,
                        // `clampMargin` px from its edge, `scaleAt` /
                        // `scaleMin` / `scaleMax` distance scaling,
                        // `maxDistance` depth culling, `lod` `[near, far]`.
                        "lift" | "occlusion" | "declutter" | "priority" | "clamp"
                        | "clampMargin" | "scaleAt" | "scaleMin" | "scaleMax" | "maxDistance"
                        | "lod" => {
                            let a = self.els[e as usize]
                                .anchor
                                .get_or_insert_with(|| Box::new(super::Anchor::new()));
                            let v = value.trim();
                            match name {
                                "lift" => {
                                    a.lift = [0.0; 3];
                                    for (l, t) in a
                                        .lift
                                        .iter_mut()
                                        .zip(v.split([',', ' ']).filter(|t| !t.is_empty()))
                                    {
                                        *l = t.parse().unwrap_or(0.0);
                                    }
                                }
                                "occlusion" => {
                                    a.occlusion = match v {
                                        "fade" => super::Occlusion::Fade,
                                        "hide" => super::Occlusion::Hide,
                                        _ => super::Occlusion::None,
                                    }
                                }
                                "declutter" => {
                                    a.declutter = match v {
                                        "stack" => super::Declutter::Stack,
                                        "hide" => super::Declutter::Hide,
                                        _ => super::Declutter::None,
                                    }
                                }
                                "priority" => a.priority = v.parse().unwrap_or(0.0),
                                "clamp" => a.clamp = v == "edge",
                                "clampMargin" => {
                                    a.margin = v.parse().unwrap_or(super::CLAMP_MARGIN)
                                }
                                "scaleAt" => {
                                    a.scale_at = v
                                        .parse::<f32>()
                                        .ok()
                                        .filter(|n| n.is_finite() && *n > 0.0)
                                        .unwrap_or(0.0);
                                }
                                "scaleMin" => {
                                    a.scale_min = v
                                        .parse::<f32>()
                                        .ok()
                                        .filter(|n| n.is_finite() && *n > 0.0)
                                        .unwrap_or(0.5);
                                }
                                "scaleMax" => {
                                    a.scale_max = v
                                        .parse::<f32>()
                                        .ok()
                                        .filter(|n| n.is_finite() && *n > 0.0)
                                        .unwrap_or(1.5);
                                }
                                "maxDistance" => {
                                    a.max_distance = v
                                        .parse::<f32>()
                                        .ok()
                                        .filter(|n| n.is_finite() && *n > 0.0)
                                        .unwrap_or(0.0);
                                }
                                _ => {
                                    a.lod = [0.0; 2];
                                    for (slot, tok) in a
                                        .lod
                                        .iter_mut()
                                        .zip(v.split([',', ' ']).filter(|t| !t.is_empty()))
                                    {
                                        *slot = tok
                                            .parse::<f32>()
                                            .ok()
                                            .filter(|n| n.is_finite() && *n > 0.0)
                                            .unwrap_or(0.0);
                                    }
                                }
                            }
                            self.paint = true;
                        }
                        // The box image's own props over its classes':
                        // `src` a texture handle, the wipe a number per tick.
                        "src" => self.set_src(e, value),
                        "wipe" | "wipeStart" | "wipeCcw" => {
                            let props = self.els[e as usize].image.get_or_insert_with(|| {
                                Box::new(super::ImageProps {
                                    handle: f64::NAN,
                                    ..Default::default()
                                })
                            });
                            let number = value.parse::<f32>().ok().filter(|n| n.is_finite());
                            match name {
                                "wipe" => {
                                    let next = number.map(|w| w.clamp(0.0, 1.0));
                                    props.wipe = next;
                                    let target = next.unwrap_or(1.0);
                                    if self.els[e as usize].record.skin.image.wipe != target {
                                        let mut next_rec = self.els[e as usize].record.clone();
                                        next_rec.skin.image.wipe = target;
                                        let old = std::mem::replace(
                                            &mut self.els[e as usize].record,
                                            next_rec,
                                        );
                                        let el = &self.els[e as usize];
                                        if el.entering.is_none() {
                                            self.motion.restyle(el.node, &old, &el.record);
                                        }
                                    }
                                }
                                "wipeStart" => props.wipe_start = number,
                                _ => {
                                    props.wipe_ccw = (!value.is_empty())
                                        .then_some(!matches!(value, "false" | "0"))
                                }
                            }
                            self.paint = true;
                        }
                        "name" if self.els[e as usize].kind == Kind::Icon => {
                            self.set_text(e, value);
                        }
                        // `ruby`: a span's annotation (tree/para.rs).
                        "ruby" if self.els[e as usize].kind == Kind::Text => {
                            if *self.els[e as usize].ruby != *value {
                                self.els[e as usize].ruby = value.into();
                                self.para_touch(e);
                            }
                        }
                        // `name`: what `anchor` and `spotlight` find it by.
                        "name" => {
                            self.input.per[e as usize].name = value.trim().into();
                            self.paint = true;
                        }
                        // `spotlight`: the named node its ground lights.
                        "spotlight" => {
                            self.input.per[e as usize].spotlight = value.trim().into();
                            if !self.spotlights.contains(&e) {
                                self.spotlights.push(e);
                            }
                            // Off until the next frame finds its node.
                            if value.trim().is_empty() {
                                self.mark(e, "lit", false);
                            }
                            self.paint = true;
                        }
                        // An accessibility state is also the class state
                        // of its name, as NSH's core sets it from a view's
                        // `accessibilityState`: `checked` true is
                        // `checked:`, "mixed" `indeterminate:`.
                        "checked" | "pressed" | "selected" | "expanded" | "busy" | "invalid" => {
                            let v = value.trim().trim_matches('"');
                            let on = matches!(v, "true" | "1");
                            match name {
                                "checked" => {
                                    self.set_state(e, CHECKED, on);
                                    self.set_state(e, INDETERMINATE, v == "mixed");
                                }
                                "pressed" => self.set_state(e, ARIA_PRESSED, on),
                                "selected" => self.set_state(e, SELECTED, on),
                                "expanded" => self.set_state(e, EXPANDED, on),
                                "busy" => self.set_state(e, BUSY, on),
                                _ => self.set_state(e, INVALID, on),
                            }
                        }
                        // A `data-*` attribute, on unless empty or false:
                        // the classes' `data-*:` variants read it here,
                        // their `group-data-*:` ones in its group's nodes.
                        n if n.starts_with("data-") => {
                            let n = &n["data-".len()..];
                            let on = !matches!(value, "" | "false");
                            let attrs = &mut self.els[e as usize].attrs;
                            match (attrs.binary_search_by(|a| (**a).cmp(n)), on) {
                                (Err(at), true) => attrs.insert(at, n.into()),
                                (Ok(at), false) => drop(attrs.remove(at)),
                                _ => {}
                            }
                            self.data_dirty = true;
                        }
                        // The rest are read by their own heads: roles and
                        // labels by a11y.
                        _ => {}
                    }
                }
                // listen
                7 => {
                    let e = self.op(r.u32()?)?;
                    let event = r.u8()?;
                    let on = r.u8()? != 0;
                    let Some(&ev) = Event::ALL.get(event as usize) else {
                        return Err(format!("og.ui ops: unknown event {event}"));
                    };
                    let el = &mut self.els[e as usize];
                    el.listens = if on {
                        el.listens | ev.bit()
                    } else {
                        el.listens & !ev.bit()
                    };
                    let was = el.clickable;
                    el.clickable = el.listens & (Event::Press.bit() | Event::Tap.bit()) != 0;
                    let (node, pressable) = (el.node, el.clickable);
                    self.sem.pressable(node, pressable);
                    if was != pressable {
                        // Its squash under a press is the pack's now, or no longer.
                        self.restyle(e);
                    }
                }
                // insert
                8 => {
                    let parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    let before = r.u32()?;
                    let (p, pnode) = match parent {
                        0 => (NONE, 0),
                        id => {
                            let p = self.op(id)?;
                            (p, self.els[p as usize].node)
                        }
                    };
                    let before = match before {
                        u32::MAX => None,
                        id => Some(self.els[self.op(id)? as usize].node),
                    };
                    // The paragraph it leaves and the one it joins lay out again.
                    self.para_touch(self.els[child as usize].parent);
                    self.arena
                        .attach(self.els[child as usize].node, pnode, before);
                    self.set_parent(child, p);
                    self.els[child as usize].arena_parent = p;
                    self.para_touch(child);
                    self.data_dirty = true;
                    let held = self.leaving.iter().any(|l| l.0 == child)
                        && !self.motion.exiting(self.els[child as usize].node);
                    self.leaving.retain(|l| l.0 != child);
                    let el = &self.els[child as usize];
                    self.sem.exiting(el.node, false);
                    if held {
                        // Put back while what is under it exits: they turn back.
                        let mut stack: Vec<u32> = self.els[child as usize].children.clone();
                        while let Some(d) = stack.pop() {
                            let el = &self.els[d as usize];
                            if el.dead {
                                continue;
                            }
                            stack.extend_from_slice(&el.children);
                            if self.motion.exiting(el.node) {
                                self.motion.enter(el.node, &el.record, None);
                            }
                        }
                    }
                    self.ordered = false;
                    self.paint = true;
                    self.reapply(child);
                    // Into a parent already on screen it enters from its
                    // `starting:` record; the first build appears as it is.
                    let shown = match p {
                        NONE => !self.abs.is_empty(),
                        p => self.abs.get(p as usize).is_some_and(|a| a.2),
                    };
                    let wait = if shown {
                        self.handoff(child) + self.cascade(child)
                    } else {
                        0.0
                    };
                    let el = &self.els[child as usize];
                    let from = self.classes.presence(el.class, 0).filter(|_| shown);
                    // One put back while it exits turns back from where it
                    // is, and restyles as a drawn node does.
                    if !self.motion.exiting(el.node) {
                        let el = &mut self.els[child as usize];
                        if el.entering.replace(shown).is_none() {
                            self.entering.push(child);
                        }
                    }
                    let el = &self.els[child as usize];
                    self.motion.enter_after(el.node, &el.record, from, wait);
                    if shown {
                        self.enter_subtree(child, wait);
                        if self.els[child as usize].record.glide.is_some() {
                            self.glide_from(child);
                        }
                    }
                }
                // remove
                9 => {
                    let _parent = r.u32()?;
                    let child = self.op(r.u32()?)?;
                    self.play_exit_sounds(child);
                    self.para_touch(self.els[child as usize].parent);
                    if self.els[child as usize].record.glide.is_some() {
                        self.left_glide(child);
                    }
                    let el = &self.els[child as usize];
                    // With an `exit:` record it stays drawn, not hit,
                    // until its exit settles.
                    let to = self.classes.presence(el.class, 1);
                    if self.motion.exit(el.node, &el.record, to) || self.exit_under(child) {
                        self.sem.exiting(self.els[child as usize].node, true);
                        self.leaving.push((child, false));
                        self.paint = true;
                        continue;
                    }
                    self.arena.detach(self.els[child as usize].node);
                    self.set_parent(child, NONE);
                    self.els[child as usize].arena_parent = NONE;
                    self.ordered = false;
                    self.paint = true;
                }
                // release
                10 => {
                    let e = self.op(r.u32()?)?;
                    self.para_touch(self.els[e as usize].parent);
                    self.release(e);
                    self.ordered = false;
                    self.paint = true;
                }
                // placeholder: an anchor that takes no room
                11 => {
                    let e = self.create(r.u32()?, Kind::Box, false)?;
                    self.sem
                        .create(self.els[e as usize].node, a11y::Kind::Placeholder);
                    self.els[e as usize].hidden = 1;
                    self.inherit(e);
                }
                // strings: a string table the bundle's components import,
                // for the host to register as base strings
                12 => {
                    let id = r.str()?;
                    self.strings.push(id.to_owned());
                }
                c => return Err(format!("og.ui ops: unknown op {c}")),
            }
        }
        if r.at != bytes.len() {
            return Err("og.ui ops: bytes after the last op".into());
        }
        Ok(())
    }

    /// Applies an op batch (after [`MAGIC`]) in order. A refused op stops
    /// the batch; the ones before it stay applied, and the data contexts
    /// follow them.
    pub fn apply(&mut self, bytes: &[u8]) -> Result<(), String> {
        let applied = self.apply_ops(bytes);
        self.refresh_data();
        applied
    }

    /// Writes the events since the last call, after [`EVENTS`]: a u32
    /// count, then each one's id (u32), event (u8), x and y (f32), key and
    /// value (u32 length and UTF-8).
    /// The string tables the batches since the last call named (op 12), by
    /// bundle id: the host registers each as base strings.
    pub fn take_strings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.strings)
    }

    pub fn take_events(&mut self, out: &mut Vec<u8>) {
        self.fire_scrolled();
        out.extend_from_slice(EVENTS);
        out.extend_from_slice(&(self.events.len() as u32).to_le_bytes());
        for f in self.events.drain(..) {
            out.extend_from_slice(&f.id.to_le_bytes());
            out.push(f.event as u8);
            out.extend_from_slice(&f.x.to_le_bytes());
            out.extend_from_slice(&f.y.to_le_bytes());
            for s in [&f.key, &f.value] {
                out.extend_from_slice(&(s.len() as u32).to_le_bytes());
                out.extend_from_slice(s.as_bytes());
            }
        }
    }
}
