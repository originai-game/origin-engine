//! The inspector for humans (docs/design.md §1, §3.6; roadmap G3-7): a
//! panel drawn over a development build's frame, in the engine's own
//! style, showing the world's entities by og.Parent and the selected
//! one's components, every field edited through the reflected schema.
//!
//! - **Its own UI.** An og-ui `Ui` of its own, in the default style at the
//!   mouse density, its panel the built-in document `panel.svelte` given
//!   the inspector's state as values each frame (`panel`), drawn above
//!   every layer of the game's (`emit`), so the
//!   same panel shows wherever a development run is seen: Origin Desktop
//!   through shared frames, the web page, a capture.
//! - **Its own input.** The probe's input filter ([`filter`]) hands it the
//!   events over its panel (and a press that started there until it ends),
//!   the keys while one of its fields is edited, and the toggle key; the
//!   game sees every other event as before. The toggle is F1 (`inspect:
//!   key:<code>` sets another); when the game's action map binds that key
//!   the inspector takes it only with Ctrl.
//! - **Edits are Tune writes.** An edit is the probe's `set` (`Probe::tune`):
//!   checked against the schema, applied between ticks and remembered as
//!   tuned, so Save is `save_scene` with no writer of its own. A value the
//!   game writes back over an edit flashes "changed by the game".
//! - **A transform gizmo.** On the selected entity (`gizmo`): translate,
//!   rotate and scale handles (W, E, R; world or local axes), dragged in
//!   the game's frame; each drag is one field's `set` a frame, folded into
//!   one undo step, snapped while Ctrl is held. The selected meshes' box
//!   draws with it.
//! - **Not world state.** Nothing it holds is in a snapshot or the hash,
//!   and a publish build has none of it (the module, its action and its
//!   key are compiled out; `inspect` is then an unknown action).

mod gizmo;
mod model;
mod panel;
mod ui_debug;

pub(crate) use model::Edit;

use crate::engine::Engine;
use crate::present::extract::RenderState;
use og_ecs::{Entity, World};
use og_input::{InjectedInput, KeyAction, PointerAction, PointerButton};
use std::collections::{BTreeMap, BTreeSet};

use gizmo::Pose;
use glam::{Quat, Vec2, Vec3};

mod pointer;
mod probe;
mod requests;
mod update;

pub(crate) use pointer::*;
pub(crate) use probe::*;
pub(crate) use requests::*;
pub(crate) use update::*;

/// The overlay layer the inspector draws from: above the game's UI and
/// every screen layer lifted past it (og_render::screen_layer).
const LAYER: i32 = 1 << 30;

/// Whether this host writes the scene files itself: natively it does; a
/// web page answers the texts, which go out in the frame's `saved`.
const WRITES_FILES: bool = !cfg!(target_family = "wasm");

/// The toggle key when none is set.
pub(crate) const DEFAULT_KEY: &str = "F1";

/// A field edited this session that the game may write back over: read
/// again once a tick has run.
struct Watch {
    target: String,
    entity: Option<Entity>,
    component: String,
    field: String,
    shown: Vec<String>,
    tick: u64,
}

/// The transform gizmo's state: its mode and space, and the selected
/// entity's gizmo as the last frame placed it.
#[derive(Default)]
pub(crate) struct Tool {
    pub mode: gizmo::Mode,
    pub space: gizmo::Space,
    /// og.Camera's view and the selected entity's gizmo, last frame.
    view: Option<gizmo::View>,
    on: Option<On>,
    /// The handle under the pointer.
    hover: Option<gizmo::Handle>,
    drag: Option<Dragging>,
    /// Ctrl is held: a drag snaps.
    ctrl: bool,
}

/// The selected entity under the gizmo.
#[derive(Clone, Copy)]
struct On {
    entity: Entity,
    gizmo: gizmo::Gizmo,
    /// Its pose in the world, og.Transform's own and its parent's world pose.
    world: Pose,
    local: Pose,
    parent: Option<Pose>,
}

/// A drag in progress: written once a frame while the pointer moved.
struct Dragging {
    drag: gizmo::Drag,
    /// The view it began in (a camera that follows the entity does not
    /// feed back into it).
    view: gizmo::View,
    on: On,
    at: Vec2,
    written: Option<Vec2>,
    steps: u32,
    released: bool,
}

impl Tool {
    /// The handle at a surface point, if the gizmo shows.
    fn hit(&self, x: f32, y: f32) -> Option<gizmo::Handle> {
        let view = self.view.as_ref()?;
        self.on.as_ref()?.gizmo.hit(view, Vec2::new(x, y))
    }

    /// A press on a handle begins a drag.
    fn begin(&mut self, x: f32, y: f32) -> bool {
        let (Some(view), Some(on)) = (self.view, self.on) else {
            return false;
        };
        let at = Vec2::new(x, y);
        let Some(handle) = on.gizmo.hit(&view, at) else {
            return false;
        };
        // Scale is og.Transform's own: the drag starts from it.
        let start = (on.world.0, on.world.1, on.local.2);
        let Some(drag) = gizmo::Drag::begin(&view, on.gizmo, handle, at, start) else {
            return false;
        };
        self.drag = Some(Dragging {
            drag,
            view,
            on,
            at,
            written: None,
            steps: 0,
            released: false,
        });
        self.hover = Some(handle);
        true
    }
}

/// What the panel asked for this frame, done after it is built.
pub(crate) enum Request {
    /// A `set`: the target, its command text and the edit to remember
    /// (`merge`: one more step of the same drag, folded into the last).
    Tune {
        edit: Edit,
        merge: bool,
    },
    /// `apply_prefab` or `revert_prefab` on a prefab instance: one field
    /// (`target`, `Component.field`) or every override; `fields` are the
    /// ones it covers, whose unsaved marks it clears.
    Prefab {
        entity: Entity,
        target: Option<String>,
        fields: Vec<String>,
        apply: bool,
    },
    Undo,
    Redo,
    Save,
    Pause,
    Resume,
    Step,
    Close,
}

/// One engine's inspector.
pub(crate) struct Inspector {
    pub open: bool,
    /// The toggle key (`KeyboardEvent.code`).
    pub key: String,
    /// Its UI (taken out while the panel is built).
    ui: Option<og_ui::Ui>,
    input: og_input::Input,
    /// Text typed into it since its last frame.
    typed: Vec<og_input::TextInput>,
    /// The panel's rectangle in CSS pixels, as last laid out.
    panel: Option<[f32; 4]>,
    /// A press began on the panel and has not ended.
    captured: bool,
    /// The toggle key went down here: its key-up is the inspector's too.
    toggle_held: bool,
    /// One of its fields has the keyboard.
    editing: bool,
    /// An entity a field's link asked to select, by its probe id.
    pending_select: Option<String>,
    /// Pick mode: the next press outside the panel selects what is under it.
    picking: bool,
    pick_at: Option<(f32, f32)>,
    pub selected: Option<Entity>,
    /// The UI view (`ui_debug`): a pick of the game's UI asked for at a
    /// point, the element picked (its path), whether layout bounds are
    /// drawn, and what the view read this frame.
    ui_pick: Option<(f32, f32)>,
    ui_selected: Option<String>,
    ui_bounds: bool,
    ui_view: ui_debug::View,
    tab: usize,
    filter: String,
    collapsed: BTreeSet<u64>,
    folded: BTreeSet<String>,
    /// The text of the field being typed in: its key and its text.
    draft: Option<(String, String)>,
    history: model::History,
    /// Targets edited since the last save.
    dirty: BTreeSet<String>,
    watch: Vec<Watch>,
    /// Targets the game wrote back over an edit, until this frame.
    flash: BTreeMap<String, u64>,
    status: String,
    frames: u64,
    requests: Vec<Request>,
    /// The game binds the toggle key, so it needs Ctrl.
    needs_ctrl: bool,
    pub(crate) tool: Tool,
    /// The selected entity's prefab as last read (`Probe::entity_prefab`
    /// reads the scene's text: once per selection and after a save).
    prefab: Option<(Entity, Option<serde_json::Value>)>,
    /// Scene texts a save made on a host that writes no files, for the
    /// next frame's answer (`saved`: the web page routes them).
    pub(crate) saved: Vec<(String, String)>,
}

impl Inspector {
    fn new(ticks_per_second: u32) -> Inspector {
        Inspector {
            open: false,
            key: DEFAULT_KEY.into(),
            ui: Some(panel::ui(ticks_per_second)),
            input: og_input::Input::default(),
            typed: Vec::new(),
            panel: None,
            captured: false,
            toggle_held: false,
            editing: false,
            pending_select: None,
            picking: false,
            pick_at: None,
            selected: None,
            ui_pick: None,
            ui_selected: None,
            ui_bounds: false,
            ui_view: ui_debug::View::default(),
            tab: 0,
            filter: String::new(),
            collapsed: BTreeSet::new(),
            folded: BTreeSet::new(),
            draft: None,
            history: model::History::default(),
            dirty: BTreeSet::new(),
            watch: Vec::new(),
            flash: BTreeMap::new(),
            status: String::new(),
            frames: 0,
            requests: Vec::new(),
            needs_ctrl: false,
            tool: Tool::default(),
            prefab: None,
            saved: Vec::new(),
        }
    }

    fn over_panel(&self, x: f32, y: f32) -> bool {
        self.panel
            .is_some_and(|[px, py, w, h]| x >= px && y >= py && x < px + w && y < py + h)
    }

    /// The toggle key's name as the panel says it.
    fn key_label(&self) -> String {
        if self.needs_ctrl {
            format!("Ctrl+{}", self.key)
        } else {
            self.key.clone()
        }
    }
}

/// The engine's inspector, made on first use.
fn of(rs: &mut RenderState, tick_rate: u32) -> &mut Inspector {
    rs.inspector
        .get_or_insert_with(|| Box::new(Inspector::new(tick_rate)))
}

fn tick_rate(world: &World) -> u32 {
    let hz = og_probe::time_json(world)["rate"].as_f64().unwrap_or(60.0);
    (hz.round() as u32).max(1)
}

#[cfg(test)]
mod tests;
