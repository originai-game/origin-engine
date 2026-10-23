//! What the hosts and the other modules provide: frames ([`Capture`], the
//! renderer), the UI tree ([`UiTree`], the UI module), and the bundle swap
//! of a hot reload ([`Reloader`], the host). Each is optional: without one,
//! the members that need it refuse exactly as protocol 5 says.

use og_ecs::{Entity, World};
use serde_json::{Value, json};

/// A frame read back from the renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// The PNG file bytes.
    pub png: Vec<u8>,
}

/// What a screen point resolved to.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub entity: Entity,
    /// Two components in the 2D profile, three in 3D.
    pub world: Vec<f32>,
    /// Where a person would change it, as `player.ts::jump`.
    pub source: Option<String>,
}

/// The renderer as the probe sees it. The host implements it over og-render
/// (`Renderer` + `read_offscreen` on native, the wasm capture on the web),
/// building the frame from the world as of the current tick. Rendering never
/// feeds back into the world, so `capture` takes it shared.
pub trait Capture {
    /// The backend: `webgpu`, `webgl2`, `vulkan`, `vulkan-cpu` (a software
    /// rasterizer) or `metal` (IDL `RenderTier`).
    fn tier(&self) -> &str;
    /// The render target's size in pixels.
    fn target_size(&self) -> (u32, u32);
    /// Render the world at its current tick offscreen and read it back as a
    /// PNG of `width`×`height` (0×0 is the render target's size). `None`
    /// when no frame can be produced.
    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Frame>;
    /// Whether `capture` answers `None` until the readback lands and the
    /// frame on a later call at the same size (the web): the probe then
    /// waits for it (`screenshot`, `sweep`) instead of failing.
    fn deferred(&self) -> bool {
        false
    }
    /// Keeps the renderer's state that follows the world's time (the
    /// adapted auto exposure) with a snapshot the probe takes (`snapshot`,
    /// a sweep's start); the answer comes back to
    /// [`Capture::restore_state`] when that snapshot is restored, so the
    /// frames after it are the snapshot's. 0 keeps nothing.
    fn save_state(&mut self) -> u64 {
        0
    }
    /// Brings back what [`Capture::save_state`] answered `state` for (0, or
    /// a state no longer kept: the renderer starts that state afresh).
    fn restore_state(&mut self, state: u64) {
        let _ = state;
    }
    /// What is under a point of the play surface, in CSS pixels. `None` until
    /// the renderer provides picking: the probe answers `pick` with `null`.
    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let _ = (world, x, y);
        None
    }
    /// The frame the host's loop drew last, read back as it is in the render
    /// target: no second draw, no PNG, no JSON-RPC (`og render` records one
    /// every frame, docs/design.md §2.9). `None` before a frame was drawn or
    /// from a host that cannot read its target back synchronously (the web).
    fn frame(&mut self) -> Option<RawFrame> {
        None
    }
}

/// A frame as RGBA8 rows, top row first ([`Capture::frame`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// One UI element with its bounds, in CSS pixels (IDL `probe.UiElement`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiElement {
    /// Its stable id, as `touch_move_to` targets it.
    pub id: String,
    /// Its parent's id; empty at the root.
    pub parent: String,
    /// What made it (`label`, `button`, `icon`, `image`, ...); empty when
    /// the UI module does not say.
    pub kind: String,
    /// A disabled control.
    pub disabled: bool,
    /// Only part of it shows: a scroll container or the surface cuts it
    /// (the bounds below are the part that shows).
    pub clipped: bool,
    /// The text it shows; empty for none.
    pub text: String,
    /// Its accessibility role (`button`, `link`, `text`, `heading`,
    /// `checkbox`, `slider`, `dialog`...); empty for a presentational
    /// element (a layout container, a spacer).
    pub role: String,
    /// Its accessible name: what a screen reader says it is.
    pub name: String,
    /// Its current value as text (a slider's number, a progress's percent,
    /// a field's text); empty for none.
    pub value: String,
    /// Its states: `disabled`, `focused`, `focusable` (keys and pads can
    /// reach it now), `checked`, `selected`, `expanded`, `pressed`, `busy`.
    pub states: Vec<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiElement {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id, "parent": self.parent, "kind": self.kind, "disabled": self.disabled, "clipped": self.clipped, "text": self.text,
            "role": self.role, "name": self.name, "value": self.value, "states": self.states,
            "x": self.x, "y": self.y, "width": self.width, "height": self.height,
        })
    }
}

/// The UI tree, which the UI module implements: `dump_ui`, `touch_move_to`
/// and `move_to_text` read it.
pub trait UiTree {
    /// Every element, flattened in draw order (a later element is drawn on
    /// top of an earlier one), with its layout as of the current tick.
    fn elements(&self, world: &World) -> Vec<UiElement>;

    /// The surface the elements are laid out on: width and height in CSS
    /// pixels and device pixels per CSS pixel. `None` when unknown.
    fn surface(&self, _world: &World) -> Option<(f32, f32, f32)> {
        None
    }
}

/// A fixed UI tree: for tests, and for a host with no UI module yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticUi(pub Vec<UiElement>);

impl UiTree for StaticUi {
    fn elements(&self, _world: &World) -> Vec<UiElement> {
        self.0.clone()
    }
}

/// The element `touch_move_to` names: an exact id, or a `Parent/Child`
/// path of ids whose last segment is the element and whose earlier segments
/// are its nearest ancestors. Topmost (last drawn) first.
pub fn find_by_id<'a>(elements: &'a [UiElement], path: &str) -> Option<&'a UiElement> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let (last, ancestors) = segments.split_last()?;
    elements.iter().rev().find(|e| {
        if e.id != *last {
            return false;
        }
        let mut parent = e.parent.as_str();
        for want in ancestors.iter().rev() {
            if parent != *want {
                return false;
            }
            parent = elements
                .iter()
                .find(|p| p.id == parent)
                .map_or("", |p| p.parent.as_str());
        }
        true
    })
}

/// The element `move_to_text:[=]text[@N]` names. A plain text is a
/// substring match, `=text` the whole text; elements are searched topmost
/// first and `@N` takes the Nth match (0-based, negative from the end).
pub fn find_by_text<'a>(elements: &'a [UiElement], query: &str) -> Option<&'a UiElement> {
    let (query, nth) = match query.rsplit_once('@') {
        Some((text, n)) if n.parse::<i64>().is_ok() => (text, n.parse::<i64>().unwrap_or(0)),
        _ => (query, 0),
    };
    let (exact, text) = match query.strip_prefix('=') {
        Some(rest) => (true, rest),
        None => (false, query),
    };
    if text.is_empty() {
        return None;
    }
    let matches: Vec<&UiElement> = elements
        .iter()
        .rev()
        .filter(|e| {
            if exact {
                e.text == text
            } else {
                e.text.contains(text)
            }
        })
        .collect();
    let index = if nth < 0 {
        matches.len().checked_sub(nth.unsigned_abs() as usize)?
    } else {
        nth as usize
    };
    matches.get(index).copied()
}

/// A loaded replacement bundle: the world it builds and how long loading took.
pub struct Rebuilt {
    /// A fresh world with the new bundle's types registered and its systems
    /// added. Entities are not needed: the probe restores the old world's.
    pub world: World,
    /// Milliseconds the bundle swap took, measured by the host (the probe
    /// reads no clock).
    pub ms: f64,
}

/// The host's half of a hot reload (design §3.5): swap the game bundle.
pub trait Reloader {
    /// Load the changed bundle into a new world. `Err` names why it failed to
    /// load; the old bundle and world keep running.
    fn rebuild(&mut self, current: &World) -> Result<Rebuilt, String>;
}
