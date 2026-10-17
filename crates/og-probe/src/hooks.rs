//! What the hosts and the other modules provide: frames ([`Capture`], the
//! renderer), the UI tree ([`UiTree`], the UI module), and the bundle swap
//! of a hot reload ([`Reloader`], the host). Each is optional: without one,
//! the members that need it refuse exactly as protocol 5 says.

use og_ecs::{Entity, World};
use serde_json::{json, Value};

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
    /// The backend: `webgpu`, `webgl2`, `vulkan` or `metal` (IDL `RenderTier`).
    fn tier(&self) -> &str;
    /// The render target's size in pixels.
    fn target_size(&self) -> (u32, u32);
    /// Render the world at its current tick offscreen and read it back as a
    /// PNG of `width`×`height` (0×0 is the render target's size). `None`
    /// when no frame can be produced.
    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Frame>;
    /// What is under a point of the play surface, in CSS pixels. `None` until
    /// the renderer provides picking: the probe answers `pick` with `null`.
    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let _ = (world, x, y);
        None
    }
}

/// One UI element with its bounds, in CSS pixels (IDL `probe.UiElement`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiElement {
    /// Its stable id, as `touch_move_to` targets it.
    pub id: String,
    /// Its parent's id; empty at the root.
    pub parent: String,
    /// The text it shows; empty for none.
    pub text: String,
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
            "id": self.id, "parent": self.parent, "text": self.text,
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
