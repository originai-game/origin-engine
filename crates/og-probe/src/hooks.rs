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
    /// rasterizer), `metal` or `dx12` (IDL `RenderBackend`).
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
    /// [`Capture::capture`] from another camera (`capture_views`): the
    /// world as of its tick drawn from `view` in place of og.Camera, which
    /// is never written. Levels of detail and what is extracted stay the
    /// main camera's. `None` when no frame can be produced (or, a deferred
    /// host, until its readback lands: asked again with the same view).
    fn capture_from(
        &mut self,
        world: &World,
        view: &Viewpoint,
        width: u32,
        height: u32,
    ) -> Option<Frame> {
        let _ = (world, view, width, height);
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

/// A camera to capture from (`capture_views`; IDL `probe.Viewpoint`), in
/// world units.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewpoint {
    pub position: [f32; 3],
    pub target: [f32; 3],
    /// Default `[0, 1, 0]`.
    pub up: [f32; 3],
    /// Vertical field of view in degrees (perspective); ignored with `height`.
    pub fov: f32,
    /// An orthographic view's height in world units; `None` is perspective.
    pub height: Option<f32>,
    /// Near and far planes; `None` keeps og.Camera's.
    pub near: Option<f32>,
    pub far: Option<f32>,
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
    /// For an og.Skeleton: its skin's joints as this frame poses them
    /// (`describe`'s `joints`); empty otherwise.
    pub joints: Vec<ShownJoint>,
}

/// One joint of a described skeleton on screen.
#[derive(Clone, Debug, PartialEq)]
pub struct ShownJoint {
    /// Its humanoid role (og.anim.humanoid's VRM name: `head`, `hips`,
    /// `leftFoot`, `rightHand` ...) when the rig is recognised, else its
    /// node's name.
    pub name: String,
    /// Whether `name` is a humanoid role.
    pub humanoid: bool,
    /// Where it lands on the surface, in CSS pixels (outside it when off
    /// screen).
    pub point: [f32; 2],
    /// From the camera along the view, in world units (negative behind it).
    pub depth: f32,
    /// On screen, in front of the camera and hidden by nothing else drawn
    /// or by opaque UI (the skeleton's own surface does not hide it).
    pub visible: bool,
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

/// Why an entity is or is not drawn by the main camera (IDL
/// `probe.DrawReason`), in the order the host decides it: the first that
/// holds is the reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawReason {
    /// A drawable of it survives the camera's cull.
    Drawn,
    /// Its model is drawn by an HLOD proxy in its place.
    Hlod,
    /// A component of it that draws has `visible` false (the detail names
    /// the field, `og.Model.visible`).
    Hidden,
    /// Its model has not loaded yet (the detail names the asset).
    Loading,
    /// Every drawable of it is off the camera's layers.
    Layer,
    /// Every drawable of it lies outside the camera's view, past near or
    /// far included.
    OutsideView,
    /// A drawable of it lies inside the camera's view, behind what the
    /// occluders drew (the GPU cull's occlusion test), and none is drawn.
    Occluded,
    /// It is fading in (`og.render.reveal`) and none of it shows yet.
    FadingIn,
    /// It draws through a path the cull record does not follow (particles,
    /// lines, text, a tilemap; the detail names the component): whether
    /// it shows is not known.
    Untracked,
    /// It has nothing to draw: no component that draws.
    NothingToDraw,
}

impl DrawReason {
    /// The reason code: `drawn`, `hlod`, `hidden`, `loading`, `layer`,
    /// `outside_view`, `occluded`, `fading_in`, `untracked`,
    /// `nothing_to_draw`.
    pub fn code(self) -> &'static str {
        match self {
            DrawReason::Drawn => "drawn",
            DrawReason::Hlod => "hlod",
            DrawReason::Hidden => "hidden",
            DrawReason::Loading => "loading",
            DrawReason::Layer => "layer",
            DrawReason::OutsideView => "outside_view",
            DrawReason::Occluded => "occluded",
            DrawReason::FadingIn => "fading_in",
            DrawReason::Untracked => "untracked",
            DrawReason::NothingToDraw => "nothing_to_draw",
        }
    }

    /// Whether the camera draws it: `Some(true)` for drawn and hlod, `None`
    /// when not known (untracked).
    pub fn visible(self) -> Option<bool> {
        match self {
            DrawReason::Drawn | DrawReason::Hlod => Some(true),
            DrawReason::Untracked => None,
            _ => Some(false),
        }
    }
}

/// One entity in the main camera's cull of the frame at the current tick
/// (the `visible` member, `dump_entity`'s `visible`, `dump_drawn`).
#[derive(Clone, Debug, PartialEq)]
pub struct Drawn {
    pub entity: Entity,
    pub reason: DrawReason,
    /// What the reason names, empty for none: the field that hides it, the
    /// asset loading, the component not tracked.
    pub detail: String,
    /// Its drawables (instances, skinned instances, sprites, terrains) that
    /// survived the cull, and those culled.
    pub drawn: u32,
    pub culled: u32,
}

impl Drawn {
    /// `{visible, reason, detail?, drawn, culled}` (IDL `probe.Visibility`).
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "visible": self.reason.visible(),
            "reason": self.reason.code(),
            "drawn": self.drawn,
            "culled": self.culled,
        });
        if !self.detail.is_empty() {
            v["detail"] = json!(self.detail);
        }
        v
    }
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
    /// What the main camera's cull made of the last frame drawn, as the
    /// culls that drew it decided: every entity the renderer drew, culled
    /// or held back (`drawn`, `hlod`, `loading`, `layer`, `outside_view`,
    /// `occluded`, `fading_in`, `untracked` for a wireframe), any order.
    /// The probe answers every other entity from its components
    /// (`hidden`, `untracked`, `nothing_to_draw`). `None` when the host
    /// has no renderer or has drawn no frame.
    fn drawn(&self, world: &World) -> Option<Vec<Drawn>> {
        let _ = world;
        None
    }
}

/// Source provenance and memory-only prefab resets supplied by the host.
/// This hook never edits an author's source files.
pub trait SceneAuthor {
    /// Checked scene/prefab target accepted by `og set` and Stage source edits:
    /// `{scene, entity, component, field}`. None for fields without an authored
    /// declaration. Computed declarations may resolve but refuse source edits.
    fn authored_target(&self, world: &World, entity: Entity, parameter: &str) -> Option<Value> {
        let _ = (world, entity, parameter);
        None
    }

    /// Where an entity is declared: `content/scenes/main.ts:12`, the
    /// captured expression in its scene, or for one the game
    /// spawned at run time the line that spawned it (SpawnDesc.source:
    /// `src/enemies.ts:42`). `None` when neither is known (a publish build,
    /// a host that does not know).
    fn source(&self, world: &World, entity: Entity) -> Option<String> {
        let _ = (world, entity);
        None
    }

    /// Reset selected fields to their cooked prefab values in memory.
    /// Authored overrides and source files remain unchanged.
    fn prefab(&mut self, world: &mut World, edit: &PrefabEdit) -> Result<(), String> {
        let _ = (world, edit);
        Err("this host has no prefab reset".into())
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

/// One memory-only `revert_prefab`.
#[derive(Clone, Debug, PartialEq)]
pub struct PrefabEdit {
    pub entity: Entity,
    /// `Component` or `Component.field`; `None`: every override.
    pub target: Option<String>,
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
        e.kind != "part"
            && x >= e.x
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
