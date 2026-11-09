//! What the hosts and the other modules provide: frames ([`Capture`], the
//! renderer), the UI tree ([`UiTree`], the UI module), and the bundle swap
//! of a hot reload ([`Reloader`], the host). Each is optional: without one,
//! the members that need it refuse exactly as protocol 5 says.

use og_ecs::{Entity, World};
use serde_json::{Map, Value, json};

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
    /// Whether a renderer is attached. A headless host may still checkpoint
    /// presentation state through this hook without producing frames.
    fn has_renderer(&self) -> bool {
        true
    }
    /// The backend: `webgpu`, `vulkan`, `vulkan-cpu` (a software
    /// rasterizer), `metal` or `dx12` (IDL `RenderTier`).
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
    /// Why the last `capture` answered `None`, added to the probe's
    /// "the renderer produced no frame"; `None` when it does not say.
    fn capture_error(&self) -> Option<String> {
        None
    }
    /// Keeps the presentation state (docs/design.md §2.4: what frames draw
    /// that follows the ticks, such as the particles' ticks and the adapted
    /// auto exposure) with a snapshot the probe takes (`snapshot`, a
    /// sweep's start) as its presentation checkpoint; the answer comes back
    /// to [`Capture::restore_state`] when that snapshot is restored, so the
    /// frames after it are the snapshot's. 0 keeps nothing.
    fn save_state(&mut self) -> u64 {
        0
    }
    /// Refuse an incompatible presentation checkpoint before the world changes.
    fn validate_restore(&self, _state: u64) -> og_core::Result<()> {
        Ok(())
    }
    /// Brings back what [`Capture::save_state`] answered `state` for (0, or
    /// a state no longer kept: the renderer starts that state afresh).
    fn restore_state(&mut self, state: u64) {
        let _ = state;
    }
    /// The presentation hash of the state [`Capture::save_state`] keeps, as
    /// it is now: equal on one host and backend for the same frames
    /// (`snapshot`'s `presentation`); `None` without one.
    fn presentation_hash(&self) -> Option<u64> {
        None
    }
    /// How the renderer last drew an entity, for `dump_entity`'s `render`
    /// (an og.Model's level of detail: `{ "lod": 1 }`); `None` leaves it
    /// out (an entity not drawn, or no renderer).
    fn entity_render(&self, world: &World, entity: Entity) -> Option<Value> {
        let _ = (world, entity);
        None
    }
    /// What is under a point of the play surface, in CSS pixels. `None` until
    /// the renderer provides picking: the probe answers `pick` with `null`.
    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let _ = (world, x, y);
        None
    }
    /// The frame the host's loop drew last, read back as it is in the render
    /// target: no second draw, no PNG, no JSON-RPC (`og render` records one
    /// every frame, docs/design.md §2.9). `None` before a frame was drawn,
    /// or (a [`Capture::deferred`] host: the web) until the readback of the
    /// frame drawn last has landed: the first call there turns the keeping
    /// of presented frames on, so the reader asks again before the next
    /// frame, as a deferred `capture` is asked again.
    fn frame(&mut self) -> Option<RawFrame> {
        None
    }
    /// `teleport` moved `entity`: a renderer that draws between ticks
    /// draws it where it landed rather than gliding there.
    fn teleported(&mut self, world: &World, entity: Entity) {
        let _ = (world, entity);
    }
}

/// A frame as RGBA8 rows, top row first ([`Capture::frame`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// One entity a frame shows, as the renderer's entity-id pass saw it
/// (`describe`; IDL `probe.DescribedEntity`). Screen values are in CSS
/// pixels of the play surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub entity: Entity,
    /// The box of its visible pixels: `[x, y, width, height]`.
    pub rect: [f32; 4],
    /// The box of what it covers with nothing in front of it.
    pub drawn: [f32; 4],
    /// A point that shows it, nearest the box's centre (`pick` answers the
    /// entity there).
    pub point: [f32; 2],
    /// From the camera to its nearest visible point, along the view, in
    /// world units.
    pub depth: f32,
    /// Its visible share of the surface, 0 to 1.
    pub coverage: f32,
    /// The share of what it covers alone that something else hides, 0 to 1.
    pub occlusion: f32,
    /// What draws it: `model`, `mesh`, `skinned`, `sprite`, and the model's
    /// asset path, such as `models/walker.glb`.
    pub labels: Vec<String>,
}

/// The renderer's view of the frame at the current tick (`describe`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seen {
    /// The entity-id pass's size in pixels (a quarter of the frame's).
    pub pass: (u32, u32),
    /// The camera as drawn: position, target, up, projection, fov or height, near, far.
    pub camera: Value,
    /// Every entity with a visible pixel, any order (`describe` sorts them).
    pub entities: Vec<Shown>,
    /// The secondary cameras (og.CameraView, og.Mirror), each with what its
    /// own id pass shows on its layers, in the frame's view order.
    pub views: Vec<SeenView>,
}

/// One secondary camera's view of the frame (`describe`; IDL
/// `probe.DescribedView`).
#[derive(Clone, Debug, PartialEq)]
pub struct SeenView {
    /// The og.CameraView or og.Mirror entity.
    pub entity: Entity,
    /// An og.Mirror (else an og.CameraView).
    pub mirror: bool,
    /// The pixels its image covers: its target's, or its viewport's rect.
    pub size: (u32, u32),
    /// Every entity with a visible pixel in it; screen values in the
    /// image's pixels.
    pub entities: Vec<Shown>,
}

/// The frame described as data and points resolved to entities, over the
/// renderer's scene at the current tick: the host implements it with
/// og-render's entity-id pass (`render_ids`), which needs no GPU, so it
/// answers in a host without a renderer too.
pub trait SceneView {
    /// What the frame shows at the current tick; `None` when the host has
    /// no scene to describe. `cover` are the rects the UI draws opaque over
    /// the main camera's frame (`[x, y, width, height]`, CSS pixels): what
    /// lies under them is hidden, as another object in front would hide it.
    fn seen(&mut self, world: &World, cover: &[[f32; 4]]) -> Option<Seen>;
    /// What is under a point of the play surface, in CSS pixels.
    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick>;
}

/// One scene file written back (`save_scene`).
#[derive(Clone, Debug, PartialEq)]
pub struct SavedFile {
    /// The scene's source, project-relative (`assets/scenes/main.ogt`).
    pub source: String,
    /// Where the text was written: the source itself, or empty when only
    /// returned (the caller writes it).
    pub written: String,
    /// The whole new text.
    pub text: String,
    /// The 1-based lines of the new text that changed or were added.
    pub lines: Vec<usize>,
}

/// Every scene the world was assembled from, written back (`save_scene`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedScene {
    /// The boot scene's first (it holds the resources and the spawned
    /// section), then each scene og.world.instantiate made, once each.
    pub files: Vec<SavedFile>,
    /// Tuned fields that were not written, each with its reason (an entity
    /// no scene made and the spawned section was not asked for, a
    /// resource no scene sets where none takes new ones).
    pub skipped: Vec<String>,
    /// Entities written under the boot scene's spawned section.
    pub spawned: usize,
}

/// The host's half of `save_scene`: the scenes the world was assembled
/// from, rewritten line by line from the world.
pub trait SceneWriter {
    /// Rewrites each scene's text from `world`: the lines of its entities'
    /// and resources' fields whose values changed, keeping every other
    /// line, and a line or block for each `tuned` field
    /// (`<entity>.<Component>.<field>` or `<Resource>.<field>`) the text
    /// does not have. With `spawned`, the entities the game spawned at run
    /// time are written under the boot scene's marked section. With
    /// `write`, each text replaces its source file; otherwise they are only
    /// returned.
    fn save(
        &mut self,
        world: &World,
        tuned: &[String],
        write: bool,
        spawned: bool,
    ) -> Result<SavedScene, String>;

    /// Where an entity is declared: `scenes/main.ogt:12`, the line of its
    /// `entity` in the scene it was assembled from. `None` for one the game
    /// spawned at run time, or a host that does not know.
    fn source(&self, world: &World, entity: Entity) -> Option<String> {
        let _ = (world, entity);
        None
    }

    /// `apply_prefab` and `revert_prefab` (`docs/design.md` §2.5): the
    /// overrides of an entity placed from a prefab moved into the prefab's
    /// text (`edit.apply`) or dropped with the live values set back to the
    /// prefab's. The files written (the scene's and, applying, the
    /// prefab's).
    fn prefab(&mut self, world: &mut World, edit: &PrefabEdit) -> Result<SavedScene, String> {
        let _ = (world, edit);
        Err("this host has no prefab write-back".into())
    }

    /// The asset id of the prefab an og.PrefabLink's `prefab` handle names
    /// (`describe`'s `prefabs`).
    fn prefab_id(&self, handle: u64) -> Option<String> {
        let _ = handle;
        None
    }

    /// `dump_entity`'s `prefab`: where a placed entity came from and what
    /// its instance overrides; `None` leaves it out.
    fn entity_prefab(&self, world: &World, entity: Entity) -> Option<Value> {
        let _ = (world, entity);
        None
    }
}

/// One `apply_prefab` or `revert_prefab`.
#[derive(Clone, Debug, PartialEq)]
pub struct PrefabEdit {
    pub entity: Entity,
    /// `Component` or `Component.field`; `None`: every override.
    pub target: Option<String>,
    /// Apply into the prefab (else revert to it).
    pub apply: bool,
    /// Apply into a variant's base instead of the variant.
    pub base: bool,
    /// Write the files (else only return the texts).
    pub write: bool,
}

/// One UI element with its bounds, in CSS pixels (IDL `probe.UiElement`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiElement {
    /// Its stable id, as `touch_move_to` targets it.
    pub id: String,
    /// Its parent's id; empty at the root.
    pub parent: String,
    /// Its full path (`Shop/Items/Card#2/Buy`): its id after every
    /// ancestor's. Empty when the UI module does not say; `find_by_id` then
    /// falls back to the ids and parents alone.
    pub path: String,
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
    /// It draws an opaque background over its whole box (its fill's alpha
    /// times its opacity is 1): what is under it does not show, so
    /// `describe` counts it as hiding the scene there.
    pub opaque: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// What the UI module knows of it beyond the above, each an optional
    /// field of its JSON (docs/ui.md §6): `layer` (the screen whose
    /// layer it is in), `source` (`ui/shop.ogt:42`, the document line that
    /// built it), `bindings` (`[{prop, expr, value}]`), `visualStates`
    /// (`{group: state}`), `edits` (`{prop: value}`, the probe's
    /// presentation edits on it), `motion` (`{moving, plays, leaving}`),
    /// `slot` (its art slot's spec, or `{"kind": "code"}` for a slot code
    /// fills). Empty: none of them.
    pub detail: Map<String, Value>,
}

impl UiElement {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "id": self.id, "parent": self.parent, "kind": self.kind, "disabled": self.disabled, "clipped": self.clipped, "text": self.text,
            "role": self.role, "name": self.name, "value": self.value, "states": self.states,
            "opaque": self.opaque, "x": self.x, "y": self.y, "width": self.width, "height": self.height,
        });
        if let Value::Object(o) = &mut v {
            // `path` (optional): the element's full path, absent when unknown.
            if !self.path.is_empty() {
                o.insert("path".into(), json!(self.path));
            }
            for (k, x) in &self.detail {
                o.entry(k.clone()).or_insert_with(|| x.clone());
            }
        }
        v
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

    /// The name of the UI style the game configured: empty for the
    /// engine's default style. `None` when unknown.
    fn style(&self, _world: &World) -> Option<String> {
        None
    }

    /// The UI's state beyond its elements, as optional fields of
    /// `dump_ui` (docs/ui.md §6): `layers` (the layer stack, bottom
    /// first), `motion` (every element in a visual state or with motion
    /// in flight), `actions` (the UI actions, the device used last, the
    /// action bar), `documents` (the documents loaded), `edits` (the
    /// probe's presentation edits). `None`: none.
    fn extra(&self, _world: &World) -> Option<Map<String, Value>> {
        None
    }

    /// The screens shown now, bottom of the layer stack first: what the
    /// run timeline compares after each tick (its `screen` events). Cheap,
    /// as it is read every tick; empty when the UI has no screens.
    fn screens(&self, _world: &World) -> Vec<String> {
        Vec::new()
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
///
/// An element with its full [`UiElement::path`] matches when the path ends
/// with the asked one at a segment boundary, so `A/Row/Buy` never resolves
/// to the `Buy` under `B/Row` however the ids repeat. One without a path
/// (a module that does not say) is matched by walking its parents' ids.
pub fn find_by_id<'a>(elements: &'a [UiElement], path: &str) -> Option<&'a UiElement> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let (last, ancestors) = segments.split_last()?;
    let asked = segments.join("/");
    // The element at that whole path wins over a deeper one whose path
    // ends with it (an anchor `Map` over a component's own `Bezel/Map`).
    if let Some(e) = elements.iter().rev().find(|e| e.path == asked) {
        return Some(e);
    }
    elements.iter().rev().find(|e| {
        if e.id != *last {
            return false;
        }
        if !e.path.is_empty() {
            return e.path == asked
                || e.path
                    .strip_suffix(asked.as_str())
                    .is_some_and(|head| head.ends_with('/'));
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

/// The UI element `pick` answers for a point of the play surface (CSS px):
/// the topmost (last drawn) one under it that shows something or takes
/// input: a text, a role, a state, an opaque fill or a style part. A layout
/// container that draws nothing (a HUD's full-screen root) lets the point
/// through to what is under it, and to the scene.
pub fn pick_ui(elements: &[UiElement], x: f32, y: f32) -> Option<&UiElement> {
    const LAYOUT: [&str; 6] = ["layer", "root", "row", "column", "scroll", "stack"];
    elements.iter().rev().find(|e| {
        x >= e.x
            && x < e.x + e.width
            && y >= e.y
            && y < e.y + e.height
            && (!LAYOUT.contains(&e.kind.as_str())
                || !e.text.is_empty()
                || !e.role.is_empty()
                || !e.states.is_empty()
                || e.opaque
                || e.detail.contains_key("part"))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn el(id: &str, parent: &str, path: &str, x: f32) -> UiElement {
        UiElement {
            id: id.into(),
            parent: parent.into(),
            path: path.into(),
            x,
            ..UiElement::default()
        }
    }

    /// Same-named siblings under different parents: `A/Row/Buy` is A's
    /// button even though B's `Row` and `Buy` are drawn later (on top) and
    /// the first `Row` by id is A's (U2's finding: the ancestors were
    /// resolved by bare id).
    #[test]
    fn a_path_resolves_through_its_own_ancestors() {
        let elements = vec![
            el("A", "", "A", 0.0),
            el("Row", "A", "A/Row", 0.0),
            el("Buy", "Row", "A/Row/Buy", 1.0),
            el("B", "", "B", 0.0),
            el("Row", "B", "B/Row", 0.0),
            el("Buy", "Row", "B/Row/Buy", 2.0),
        ];
        assert_eq!(find_by_id(&elements, "A/Row/Buy").map(|e| e.x), Some(1.0));
        assert_eq!(find_by_id(&elements, "B/Row/Buy").map(|e| e.x), Some(2.0));
        // Nearest ancestors only: a suffix at a segment boundary.
        assert_eq!(find_by_id(&elements, "Row/Buy").map(|e| e.x), Some(2.0));
        assert_eq!(find_by_id(&elements, "w/Buy").map(|e| e.x), None);
        assert_eq!(find_by_id(&elements, "C/Row/Buy").map(|e| e.x), None);
    }

    /// A whole path names its own element even when a later one, drawn on
    /// top, ends with the same segments (U8's anchor `Map` holding a
    /// minimap whose texture box is `Track/Bezel/Map`).
    #[test]
    fn a_whole_path_wins_over_a_deeper_suffix() {
        let elements = vec![
            el("Map", "", "Map", 1.0),
            el("Track", "Map", "Map/Track", 2.0),
            el("Bezel", "Track", "Map/Track/Bezel", 3.0),
            el("Map", "Bezel", "Map/Track/Bezel/Map", 4.0),
        ];
        assert_eq!(find_by_id(&elements, "Map").map(|e| e.x), Some(1.0));
        assert_eq!(find_by_id(&elements, "Map/Track").map(|e| e.x), Some(2.0));
        assert_eq!(find_by_id(&elements, "Bezel/Map").map(|e| e.x), Some(4.0));
    }
}
