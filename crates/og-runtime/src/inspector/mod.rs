//! The inspector for humans (docs/design.md §1, §3.6; roadmap G3-7): a
//! panel drawn over a development build's frame, in the engine's own
//! style, showing the world's entities by og.Parent and the selected
//! one's components, every field edited through the reflected schema.
//!
//! - **Its own UI.** An og-ui `Ui` of its own, in the default style at the
//!   mouse density, drawn above every layer of the game's (`emit`), so the
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
use crate::extract::RenderState;
use og_ecs::{Entity, World};
use og_input::{InjectedInput, KeyAction, PointerAction, PointerButton};
use std::collections::{BTreeMap, BTreeSet};

use gizmo::Pose;
use glam::{Quat, Vec2, Vec3};

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
        let theme = og_ui::Theme {
            density: og_ui::DensityMode::Mouse,
            ..og_ui::Theme::default()
        };
        Inspector {
            open: false,
            key: DEFAULT_KEY.into(),
            ui: Some(og_ui::Ui::new(og_ui::UiDesc {
                theme,
                ticks_per_second,
                fonts: og_ui::Fonts::default(),
            })),
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

/// The probe's input filter: true takes the event from the game.
pub(crate) fn filter(engine_id: u32, world: &World, event: &InjectedInput) -> bool {
    let Ok(eng) = crate::engine::engine(engine_id) else {
        return false;
    };
    let needs_ctrl = |key: &str| {
        eng.state
            .try_borrow()
            .ok()
            .and_then(|st| st.actions.clone())
            .is_some_and(|a| model::game_binds(&og_input::actions::lock(&a), key))
    };
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return false;
    };
    let rate = tick_rate(world);
    let ins = of(&mut rs, rate);
    if let InjectedInput::Key {
        key,
        action,
        modifiers,
    } = event
    {
        ins.tool.ctrl = match key.as_str() {
            "ControlLeft" | "ControlRight" => *action == KeyAction::Down,
            _ => modifiers.control,
        };
    }
    match event {
        InjectedInput::Key {
            key,
            action,
            modifiers,
        } if *key == ins.key => {
            if *action == KeyAction::Up {
                return std::mem::take(&mut ins.toggle_held);
            }
            ins.needs_ctrl = needs_ctrl(key);
            if ins.needs_ctrl && !modifiers.control {
                return false;
            }
            ins.open = !ins.open;
            ins.toggle_held = true;
            true
        }
        _ if !ins.open => false,
        InjectedInput::Pointer {
            action,
            x,
            y,
            id,
            button,
            ..
        } => {
            if *id != 0 {
                return false;
            }
            // A gizmo drag has the pointer until it ends.
            if let Some(d) = ins.tool.drag.as_mut() {
                d.at = Vec2::new(*x, *y);
                if *action == PointerAction::Up {
                    d.released = true;
                }
                // Its own UI saw the press: it sees the release too.
                ins.input.apply(event);
                return true;
            }
            let over = ins.over_panel(*x, *y);
            let mut take = over || ins.captured;
            if !over && !ins.captured {
                ins.tool.hover = ins.tool.hit(*x, *y);
            }
            match action {
                PointerAction::Down => {
                    if over {
                        ins.captured = true;
                    } else if !ins.picking
                        && matches!(button, PointerButton::Left | PointerButton::None)
                        && ins.tool.begin(*x, *y)
                    {
                        // A field being edited is left as a click
                        // elsewhere leaves it.
                        ins.input.apply(event);
                        return true;
                    } else if ins.picking {
                        if ins.tab == ui_debug::TAB {
                            ins.ui_pick = Some((*x, *y));
                        } else {
                            ins.pick_at = Some((*x, *y));
                        }
                        ins.picking = false;
                        ins.captured = true;
                        take = true;
                    }
                }
                PointerAction::Up => ins.captured = false,
                _ => {}
            }
            // Its own UI sees every move (to leave a hover) and the presses
            // it takes; a press outside also reaches it, which ends a
            // field's edit as a click elsewhere does.
            ins.input.apply(event);
            take
        }
        InjectedInput::Wheel { x, y, .. } => {
            let over = ins.over_panel(*x, *y);
            if over {
                ins.input.apply(event);
            }
            over
        }
        InjectedInput::Key { .. } | InjectedInput::Text { .. } => {
            // Keys are the panel's while a field of it is edited, or while
            // the pointer rests on it (Ctrl+Z, Escape).
            let pointer = ins.input.pointer;
            let over = ins.over_panel(pointer.x, pointer.y);
            // W, E and R pick the gizmo's mode: over the panel, or anywhere
            // when the game binds none of them.
            if let InjectedInput::Key { key, action, .. } = event
                && !ins.editing
                && let Some(mode) = gizmo::Mode::of_key(key)
                && (over || !needs_ctrl(key))
            {
                if *action == KeyAction::Down {
                    ins.tool.mode = mode;
                }
                return true;
            }
            let take = ins.editing || over;
            if take {
                match event {
                    InjectedInput::Text { text, composing } => ins
                        .typed
                        .extend(og_input::TextInput::split(text, *composing)),
                    _ => {
                        ins.input.apply(event);
                    }
                }
            }
            take
        }
        _ => false,
    }
}

/// Builds the panel for this frame and does what it asked: between ticks,
/// before the frame is drawn.
pub(crate) fn frame(eng: &Engine) {
    // Closed (as in every run that never opens it), it costs nothing.
    let open = eng
        .render
        .try_borrow()
        .is_ok_and(|rs| rs.inspector.as_ref().is_some_and(|i| i.open));
    if !open {
        return;
    }
    let viewport = crate::modules::viewport_of(eng.id);
    let Ok(world) = eng.world() else {
        return;
    };
    let tick = world.time().tick;
    let paused = eng.probe.borrow().paused();
    let layouts = eng.state.borrow().layouts.clone();
    let (sounds, assets) = handle_names(eng);
    let camera = crate::api::resource_value(eng, "og.Camera").ok();
    let requests = {
        let Ok(mut rs) = eng.render.try_borrow_mut() else {
            return;
        };
        let fonts = rs.ui.as_ref().map(|u| {
            let u = u.lock();
            (u.fonts().clone(), u.icons().clone())
        });
        let game = rs.ui.clone();
        let Some(ins) = rs.inspector.as_mut().filter(|i| i.open) else {
            return;
        };
        if let Some((fonts, icons)) = fonts
            && let Some(ui) = ins.ui.as_mut()
            && *ui.fonts() != fonts
        {
            ui.set_fonts(fonts);
            ui.set_icons(icons);
        }
        ins.frames += 1;
        // The UI view reads the game's UI only while it is the tab shown.
        ins.ui_view = match game.filter(|_| ins.tab == ui_debug::TAB) {
            Some(game) => ui_debug::read(ins, &game.lock()),
            None => ui_debug::View::default(),
        };
        if ins.selected.is_some_and(|e| !world.is_alive(e)) {
            ins.selected = None;
        }
        let probe = eng.probe.borrow();
        if let Some((x, y)) = ins.pick_at.take() {
            drop(probe);
            let picked = eng
                .probe
                .borrow_mut()
                .pick(world, x, y)
                .and_then(|v| v["entity"].as_str().map(str::to_string));
            let probe = eng.probe.borrow();
            match picked.and_then(|id| probe.names().resolve(world, &id)) {
                Some(e) => {
                    ins.selected = Some(e);
                    ins.status = format!("Picked {}", probe.names().id_of(world, e));
                }
                None => ins.status = "Nothing under the pointer".into(),
            }
        }
        let probe = eng.probe.borrow();
        watch(ins, world, &layouts, tick);
        if ins.prefab.as_ref().map(|(e, _)| *e) != ins.selected {
            ins.prefab = ins.selected.map(|e| (e, probe.entity_prefab(world, e)));
        }
        let prefab = ins.prefab.take().filter(|(e, _)| Some(*e) == ins.selected);
        panel::build(
            ins,
            &panel::Ctx {
                world,
                names: probe.names(),
                layouts: &layouts,
                paused,
                tick,
                viewport,
                sounds: &sounds,
                assets: &assets,
                prefab: prefab.as_ref().and_then(|(_, v)| v.as_ref()),
            },
        );
        if ins.prefab.is_none() {
            ins.prefab = prefab;
        }
        if let Some(id) = ins.pending_select.take() {
            ins.selected = probe.names().resolve(world, &id).or(ins.selected);
            ins.tab = 0;
        }
        ins.tool.view = camera.map(|c| gizmo::view_of(&c, viewport.width, viewport.height));
        place(ins, world);
        drag(ins, world, probe.names());
        std::mem::take(&mut ins.requests)
    };
    for r in requests {
        apply(eng, r);
    }
}

/// og.Transform's pose of an entity, if it has one.
fn local_pose(world: &World, e: Entity) -> Option<Pose> {
    let id = world.id_by_name("og.Transform").ok()?;
    let schema = world.schema(id).ok()?;
    let bytes = world.get_bytes(e, id).ok()?;
    let layout = og_core::reflect::Layout::of(schema);
    let lanes = |name: &str, n: usize| -> Option<Vec<f32>> {
        let b = bytes.get(layout.range(schema.field_index(name)?))?;
        (0..n)
            .map(|k| {
                Some(f32::from_le_bytes(
                    b.get(k * 4..k * 4 + 4)?.try_into().ok()?,
                ))
            })
            .collect()
    };
    let p = lanes("position", 3)?;
    let q = lanes("quaternion", 4)?;
    let s = lanes("scale", 3)?;
    Some((
        Vec3::new(p[0], p[1], p[2]),
        Quat::from_xyzw(q[0], q[1], q[2], q[3]).normalize(),
        Vec3::new(s[0], s[1], s[2]),
    ))
}

/// The world pose of an entity's parent, as simulation composes it
/// (og_core::math::world_pose); none for a root.
fn parent_pose(world: &World, e: Entity) -> Option<Pose> {
    let p = model::parent_of(world, e)?;
    let local = local_pose(world, p)?;
    Some(og_core::math::world_pose(
        p,
        local,
        |k| model::parent_of(world, k),
        |k| local_pose(world, k),
    ))
}

/// The selected entity's gizmo this frame (none without og.Transform or a
/// camera).
fn place(ins: &mut Inspector, world: &World) {
    let tool = &mut ins.tool;
    tool.on = None;
    let (Some(e), Some(view)) = (ins.selected, tool.view) else {
        tool.hover = None;
        return;
    };
    let Some(local) = local_pose(world, e) else {
        tool.hover = None;
        return;
    };
    let parent = parent_pose(world, e);
    let world_pose = match parent {
        Some((pp, pq, ps)) => (
            pp + pq * (ps * local.0),
            (pq * local.1).normalize(),
            ps * local.2,
        ),
        None => local,
    };
    let g = gizmo::Gizmo::new(&view, tool.mode, tool.space, world_pose.0, world_pose.1);
    // What the pointer rests on, as this frame's gizmo has it (the mode
    // may have changed under a still pointer).
    let at = ins.input.pointer;
    let over = ins
        .panel
        .is_some_and(|[px, py, w, h]| at.x >= px && at.y >= py && at.x < px + w && at.y < py + h);
    tool.hover = match &tool.drag {
        Some(d) => Some(d.drag.handle),
        None if !over => g.hit(&view, Vec2::new(at.x, at.y)),
        None => None,
    };
    tool.on = Some(On {
        entity: e,
        gizmo: g,
        world: world_pose,
        local,
        parent,
    });
}

/// A drag's `set` for this frame, when the pointer moved since the last:
/// the one og.Transform field its mode writes, folded into the drag's
/// first step.
fn drag(ins: &mut Inspector, world: &World, names: &og_probe::Names) {
    let snap = ins.tool.ctrl;
    let Some(d) = ins.tool.drag.as_mut() else {
        return;
    };
    let released = d.released;
    if world.is_alive(d.on.entity)
        && (d.written != Some(d.at))
        && (d.steps > 0 || d.at != d.drag.grab)
    {
        let w = d.drag.pose(&d.view, d.at, snap);
        let local = gizmo::to_local(d.on.parent, w);
        let mode = d.drag.gizmo.mode;
        let text = |pose: Pose| match mode {
            gizmo::Mode::Translate => lanes(&pose.0.to_array()),
            gizmo::Mode::Rotate => lanes(&pose.1.to_array()),
            gizmo::Mode::Scale => lanes(&pose.2.to_array()),
        };
        let id = names.id_of(world, d.on.entity);
        let edit = Edit {
            target: format!("{id}.og.Transform.{}", mode.field()),
            before: text(d.on.local),
            after: text(local),
        };
        let merge = d.steps > 0;
        d.steps += 1;
        d.written = Some(d.at);
        ins.requests.push(Request::Tune { edit, merge });
    }
    if released {
        ins.tool.drag = None;
    }
}

/// Numbers as `set` takes a vector's text.
fn lanes(v: &[f32]) -> String {
    v.iter()
        .map(|x| format!("{x}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// The ids of the engine's loaded sounds and assets by handle, for the
/// handle fields the panel shows.
fn handle_names(eng: &Engine) -> (BTreeMap<u64, String>, BTreeMap<u64, String>) {
    let Ok(a) = eng.assets.try_borrow() else {
        return Default::default();
    };
    let sounds = a
        .sound_handles
        .iter()
        .zip(&a.sounds)
        .map(|(h, s)| (*h, s.id.clone()))
        .collect();
    let assets = a
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (crate::assets::handle(i), e.path.clone()))
        .collect();
    (sounds, assets)
}

/// Reads again the fields edited since a tick ran: one the game wrote
/// back flashes.
fn watch(
    ins: &mut Inspector,
    world: &World,
    layouts: &BTreeMap<String, crate::codec::Layout>,
    tick: u64,
) {
    let frames = ins.frames;
    ins.flash.retain(|_, until| *until > frames);
    let mut keep = Vec::new();
    for w in std::mem::take(&mut ins.watch) {
        if tick <= w.tick {
            keep.push(w);
            continue;
        }
        let now = panel::read_parts(world, layouts, w.entity, &w.component, &w.field);
        if now.as_ref().is_some_and(|n| *n != w.shown) {
            ins.flash.insert(w.target.clone(), frames + 120);
        } else if tick < w.tick + 30 {
            // The game may write it on a later tick (a system that runs
            // every few): keep looking for half a second.
            keep.push(w);
        }
    }
    ins.watch = keep;
}

fn apply(eng: &Engine, request: Request) {
    let set = |target: &str, text: &str| -> Result<(), String> {
        crate::probe::with_probe(eng, false, |p, w| p.tune(w, target, text))?
    };
    let note = |status: String| {
        if let Ok(mut rs) = eng.render.try_borrow_mut()
            && let Some(ins) = rs.inspector.as_mut()
        {
            ins.status = status;
        }
    };
    match request {
        Request::Tune { edit, merge } => match set(&edit.target, &edit.after) {
            Ok(()) => after_edit(eng, edit, merge),
            Err(e) => note(e),
        },
        Request::Undo | Request::Redo => {
            let step = {
                let Ok(mut rs) = eng.render.try_borrow_mut() else {
                    return;
                };
                let Some(ins) = rs.inspector.as_mut() else {
                    return;
                };
                if matches!(request, Request::Undo) {
                    ins.history.undo()
                } else {
                    ins.history.redo()
                }
            };
            if let Some((target, text)) = step {
                match set(&target, &text) {
                    Ok(()) => {
                        if let Ok(mut rs) = eng.render.try_borrow_mut()
                            && let Some(ins) = rs.inspector.as_mut()
                        {
                            ins.dirty.insert(target);
                        }
                    }
                    Err(e) => note(e),
                }
            }
        }
        Request::Prefab {
            entity,
            target,
            fields,
            apply,
        } => {
            let edit = og_probe::PrefabEdit {
                entity,
                target: target.clone(),
                apply,
                base: false,
                write: WRITES_FILES,
            };
            let done = crate::probe::with_probe(eng, false, |p, w| {
                let id = p.names().id_of(w, entity);
                p.prefab_edit(w, &edit).map(|saved| (id, saved))
            });
            match done.and_then(|r| r) {
                Ok((id, saved)) => {
                    let written = saved.files.iter().filter(|f| !f.written.is_empty()).count();
                    let what = target.unwrap_or_else(|| "every override".into());
                    let verb = if apply { "Applied" } else { "Reverted" };
                    let Ok(mut rs) = eng.render.try_borrow_mut() else {
                        return;
                    };
                    let Some(ins) = rs.inspector.as_mut() else {
                        return;
                    };
                    // The fields it covered are the prefab's again: no
                    // longer unsaved here.
                    let covers = |t: &str| {
                        t.strip_prefix(&format!("{id}.")).is_some_and(|rest| {
                            fields.iter().any(|f| panel::prefab_field(rest) == *f)
                        })
                    };
                    ins.dirty.retain(|t| !covers(t));
                    ins.watch.retain(|w| !covers(&w.target));
                    ins.prefab = None;
                    ins.status = if written > 0 {
                        format!("{verb} {what}: {written} files written")
                    } else {
                        hand_over(ins, &saved.files);
                        format!("{verb} {what}: the texts go to the page")
                    };
                }
                Err(e) => note(e),
            }
        }
        Request::Save => {
            let saved =
                crate::probe::with_probe(eng, false, |p, w| p.save_scene(w, WRITES_FILES, false));
            match saved.and_then(|r| r) {
                Ok(v) => {
                    let files = v["scenes"].as_array().map_or(0, Vec::len);
                    // `written` is the path written to (empty when the
                    // host writes no files), `lines` the changed lines.
                    let written = v["scenes"].as_array().map_or(0, |s| {
                        s.iter()
                            .filter(|f| f["written"].as_str().is_some_and(|w| !w.is_empty()))
                            .count()
                    });
                    let lines: usize = v["scenes"].as_array().map_or(0, |s| {
                        s.iter()
                            .map(|f| f["lines"].as_array().map_or(0, Vec::len))
                            .sum()
                    });
                    let skipped = v["skipped"].as_array().map_or(0, Vec::len);
                    let mut msg = if written > 0 {
                        format!("Saved {lines} lines to {written} of {files} scenes")
                    } else {
                        format!("{lines} lines ready; the page saves them")
                    };
                    if skipped > 0 {
                        msg.push_str(&format!(" ({skipped} not in scene text)"));
                    }
                    if let Ok(mut rs) = eng.render.try_borrow_mut()
                        && let Some(ins) = rs.inspector.as_mut()
                    {
                        if written > 0 {
                            ins.dirty.clear();
                        } else {
                            // A host that writes no files (the web page)
                            // hands the texts on: the frame's `saved`.
                            let files: Vec<og_probe::SavedFile> = v["scenes"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .map(|f| og_probe::SavedFile {
                                    source: f["source"].as_str().unwrap_or("").into(),
                                    written: String::new(),
                                    text: f["text"].as_str().unwrap_or("").into(),
                                    lines: f["lines"]
                                        .as_array()
                                        .map(|l| {
                                            l.iter()
                                                .filter_map(|n| n.as_u64())
                                                .map(|n| n as usize)
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                })
                                .collect();
                            if hand_over(ins, &files) {
                                ins.dirty.clear();
                            }
                        }
                        ins.prefab = None;
                        ins.status = msg;
                    }
                }
                Err(e) => note(e),
            }
        }
        Request::Pause => {
            eng.probe.borrow_mut().pause();
        }
        Request::Resume => {
            eng.probe.borrow_mut().resume();
        }
        Request::Step => {
            if let Err(e) = crate::engine::advance_probe(eng, 1) {
                note(e);
            }
        }
        Request::Close => {
            if let Ok(mut rs) = eng.render.try_borrow_mut()
                && let Some(ins) = rs.inspector.as_mut()
            {
                ins.open = false;
                ins.panel = None;
                ins.editing = false;
                ins.captured = false;
                ins.tool.drag = None;
                ins.tool.hover = None;
            }
        }
    }
}

/// Keeps the changed scene texts of a save on a host that wrote none, for
/// the next frame's answer; true when there was one.
fn hand_over(ins: &mut Inspector, files: &[og_probe::SavedFile]) -> bool {
    let before = ins.saved.len();
    for f in files
        .iter()
        .filter(|f| !f.lines.is_empty() && !f.source.is_empty())
    {
        ins.saved.retain(|(s, _)| *s != f.source);
        ins.saved.push((f.source.clone(), f.text.clone()));
    }
    ins.saved.len() > before || files.iter().any(|f| !f.lines.is_empty())
}

/// The scene texts the inspector saved since the last frame on a host
/// that writes no files, as the frame answer's `saved` (the web shell
/// sends them to the Stage or downloads them); none when there are none.
pub(crate) fn take_saved(rs: &mut RenderState) -> Option<serde_json::Value> {
    saved_of(rs.inspector.as_mut()?)
}

fn saved_of(ins: &mut Inspector) -> Option<serde_json::Value> {
    if ins.saved.is_empty() {
        return None;
    }
    let scenes: Vec<serde_json::Value> = std::mem::take(&mut ins.saved)
        .into_iter()
        .map(|(source, text)| serde_json::json!({ "source": source, "text": text }))
        .collect();
    Some(serde_json::json!({ "scenes": scenes }))
}

/// An edit that took: remembered for undo, marked unsaved and watched.
fn after_edit(eng: &Engine, edit: Edit, merge: bool) {
    let Ok(world) = eng.world() else {
        return;
    };
    let tick = world.time().tick;
    let layouts = eng.state.borrow().layouts.clone();
    let (entity, component, field) = {
        let probe = eng.probe.borrow();
        split_target(world, probe.names(), &edit.target)
    };
    let shown = panel::read_parts(world, &layouts, entity, &component, &field);
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let Some(ins) = rs.inspector.as_mut() else {
        return;
    };
    ins.dirty.insert(edit.target.clone());
    ins.flash.remove(&edit.target);
    ins.watch.retain(|w| w.target != edit.target);
    if let Some(shown) = shown {
        ins.watch.push(Watch {
            target: edit.target.clone(),
            entity,
            component,
            field,
            shown,
            tick,
        });
    }
    ins.status.clear();
    if merge {
        ins.history.merge(edit);
    } else {
        ins.history.push(edit);
    }
}

/// A `set` target's entity (none for a resource), component and field.
fn split_target(
    world: &World,
    names: &og_probe::Names,
    target: &str,
) -> (Option<Entity>, String, String) {
    let (head, field) = target.rsplit_once('.').unwrap_or((target, ""));
    if world
        .id_by_name(head)
        .is_ok_and(|id| world.has_resource_id(id))
    {
        return (None, head.to_string(), field.to_string());
    }
    let (entity, component) = match head.strip_prefix('#') {
        Some(rest) => {
            let mut p = rest.splitn(3, '.');
            match (p.next(), p.next(), p.next()) {
                (Some(i), Some(g), Some(c)) => (format!("#{i}.{g}"), c.to_string()),
                _ => (head.to_string(), String::new()),
            }
        }
        None => head
            .split_once('.')
            .map(|(e, c)| (e.to_string(), c.to_string()))
            .unwrap_or_default(),
    };
    (names.resolve(world, &entity), component, field.to_string())
}

/// Draws the gizmo and the selected meshes' box over the frame.
fn emit_gizmo(rs: &mut RenderState) {
    let Some(ins) = rs.inspector.as_ref().filter(|i| i.open) else {
        return;
    };
    let tool = &ins.tool;
    let (Some(view), Some(on)) = (tool.view, tool.on) else {
        return;
    };
    let Some(p) = ins.ui.as_ref().map(|u| u.theme().palette) else {
        return;
    };
    let colors = [p.danger, p.success, p.info, p.ink].map(|c| c.linear());
    let lit = tool.drag.as_ref().map(|d| d.drag.handle).or(tool.hover);
    let strokes = on.gizmo.strokes(&view, colors, lit, p.highlight.linear());
    let bits = on.entity.to_bits();
    let boxed = p.focus.linear();
    let instances: Vec<(og_render::MeshHandle, glam::Mat4)> = rs
        .ids
        .instances
        .iter()
        .zip(&rs.scene.objects.instances)
        .filter(|(e, _)| **e == bits)
        .map(|(_, i)| (i.mesh, i.transform))
        .collect();
    let mut bounds = Vec::new();
    for (mesh, m) in instances {
        if let Some((min, max)) = crate::geometry::mesh_bounds(rs, mesh) {
            bounds.extend(gizmo::box_lines(m, min, max));
        }
    }
    if !bounds.is_empty() {
        rs.scene
            .effects
            .lines
            .push(og_render::debug::segments(bounds, boxed, 1.0));
    }
    for (points, color, width) in strokes {
        rs.scene
            .effects
            .lines
            .push(og_render::debug::segments(points, color, width));
    }
}

/// Draws the panel over the frame (the last UI frame it built).
pub(crate) fn emit(rs: &mut RenderState) {
    emit_gizmo(rs);
    let Some(ui) = rs
        .inspector
        .as_ref()
        .filter(|i| i.open)
        .and_then(|i| i.ui.as_ref())
    else {
        return;
    };
    let frame = ui.frame();
    if frame.items.is_empty() {
        return;
    }
    let (w, h) = (
        frame.viewport.width * frame.unit,
        frame.viewport.height * frame.unit,
    );
    let time = ui.seconds() as f32;
    let overlay = rs
        .scene
        .ui
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w.max(1.0), h.max(1.0), 0.0));
    let first = overlay.texts.len();
    frame.emit(&og_ui::EmitDesc { layer: LAYER, time }, overlay);
    crate::extract::draw_other_pages(overlay, first, &rs.ui_pages);
}

/// The gizmo as `dump_inspector` reports it: its mode and space, the
/// handle under the pointer or dragged, and each handle's grab point on
/// the surface (CSS px), what a script presses to drag it.
fn gizmo_json(ins: &Inspector) -> serde_json::Value {
    let t = &ins.tool;
    let handles: Vec<serde_json::Value> = match (t.view, t.on) {
        (Some(view), Some(on)) => on
            .gizmo
            .handles()
            .into_iter()
            .filter_map(|h| {
                let p = view.project(on.gizmo.grab_point(&view, h))?;
                Some(serde_json::json!({ "handle": h.name(), "x": p.x, "y": p.y }))
            })
            .collect(),
        _ => Vec::new(),
    };
    serde_json::json!({
        "mode": t.mode.name(),
        "space": t.space.name(),
        "shown": t.on.is_some(),
        "hover": t.hover.map(|h| h.name()),
        "dragging": t.drag.as_ref().map(|d| d.drag.handle.name()),
        "handles": handles,
    })
}

/// The command channel's `inspect` and `dump_inspector`.
pub(crate) fn register(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_dump(
        "dump_inspector",
        og_probe::CommandDesc {
            usage: "dump_inspector[:path]".into(),
            doc: "the inspector's panel as data: whether it is open, the selected entity, the unsaved fields, its status line and every element of its last frame with its rectangle in CSS px (what touch_move and type reach)".into(),
        },
        0,
        move |world, _args| {
            let eng = crate::engine::engine(engine_id)?;
            let rs = eng.render.try_borrow().map_err(|_| "the renderer is busy")?;
            let Some(ins) = rs.inspector.as_ref() else {
                return Ok(serde_json::json!({ "open": false, "elements": [] }));
            };
            let selected = ins
                .selected
                .filter(|e| world.is_alive(*e))
                .map(|e| og_probe::Names.id_of(world, e));
            let elements: Vec<serde_json::Value> = if ins.open {
                ins.ui
                    .as_ref()
                    .map(|u| u.probe_elements().iter().map(|e| e.to_json()).collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            Ok(serde_json::json!({
                "open": ins.open,
                "key": ins.key_label(),
                "selected": selected,
                "ui": {
                    "selected": ins.ui_selected,
                    "bounds": ins.ui_bounds,
                    "element": ins.ui_view.element.as_ref().map(|e| serde_json::json!({
                        "path": e.path, "kind": e.kind,
                        "box": [e.border_box.x, e.border_box.y, e.border_box.width, e.border_box.height],
                        "content": [e.content.x, e.content.y, e.content.width, e.content.height],
                        "styles": e.styles.iter().map(|s| serde_json::json!({ "name": s.name, "value": s.value, "source": s.source })).collect::<Vec<_>>(),
                        "bindings": e.bindings.iter().map(|(p, x, v)| serde_json::json!({ "prop": p, "expr": x, "value": v })).collect::<Vec<_>>(),
                        "boundaries": e.boundaries,
                    })),
                },
                "dirty": ins.dirty.iter().collect::<Vec<_>>(),
                "changedByGame": ins.flash.keys().collect::<Vec<_>>(),
                "status": ins.status,
                "gizmo": gizmo_json(ins),
                "elements": elements,
            }))
        },
    );
    let _ = probe.register_command(
        "inspect",
        og_probe::CommandDesc {
            usage: "inspect[:on|off|<entity>|ui[:<path>]|key:<code>|mode:translate|rotate|scale|space:world|local]".into(),
            doc: "the inspector (development builds): open it (on, or no argument), close it (off), open it on an entity (its name or #<index>.<generation>), open its UI view (ui) on a UI element (ui:<path>, as dump_ui names it), set the key that toggles it (key:F2; F1 by default, with Ctrl when the game binds it), or the transform gizmo's mode (mode:rotate; W, E and R) and axes (space:local)".into(),
        },
        move |world, args| {
            let eng = crate::engine::engine(engine_id)?;
            let arg = args.first().map(String::as_str).unwrap_or("on");
            let selected = match arg {
                "on" | "off" => None,
                "key" | "mode" | "space" | "ui" => None,
                id => Some(
                    og_probe::Names
                        .resolve(world, id)
                        .ok_or_else(|| format!("inspect:{id}: no live entity of that id"))?,
                ),
            };
            let rate = tick_rate(world);
            let mut rs = eng.render.borrow_mut();
            let ins = of(&mut rs, rate);
            match arg {
                "on" => ins.open = true,
                "off" => ins.open = false,
                "ui" => {
                    ins.open = true;
                    ins.tab = ui_debug::TAB;
                    if args.len() > 1 {
                        ins.ui_selected = Some(args[1..].join(":"));
                    }
                }
                "key" => {
                    let code = args.get(1).map(String::as_str).unwrap_or("");
                    if !og_input::KEY_CODES.contains(&code) {
                        return Err(format!(
                            "inspect:key:{code}: not a KeyboardEvent.code (F1, F2, Backquote)"
                        ));
                    }
                    ins.key = code.to_string();
                }
                "mode" => {
                    let m = args.get(1).map(String::as_str).unwrap_or("");
                    ins.tool.mode = match m {
                        "translate" => gizmo::Mode::Translate,
                        "rotate" => gizmo::Mode::Rotate,
                        "scale" => gizmo::Mode::Scale,
                        _ => return Err(format!("inspect:mode:{m}: translate, rotate or scale")),
                    };
                }
                "space" => {
                    let m = args.get(1).map(String::as_str).unwrap_or("");
                    ins.tool.space = match m {
                        "world" => gizmo::Space::World,
                        "local" => gizmo::Space::Local,
                        _ => return Err(format!("inspect:space:{m}: world or local")),
                    };
                }
                _ => {
                    ins.open = true;
                    ins.selected = selected;
                    ins.tab = 0;
                }
            }
            Ok(())
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(source: &str, text: &str, lines: Vec<usize>) -> og_probe::SavedFile {
        og_probe::SavedFile {
            source: source.into(),
            written: String::new(),
            text: text.into(),
            lines,
        }
    }

    #[test]
    fn a_save_that_wrote_no_file_hands_its_changed_texts_to_the_next_frame_once() {
        let ins = &mut Inspector::new(60);
        // An unchanged scene is not handed on; a second save of the same
        // source replaces the first's text.
        assert!(hand_over(
            ins,
            &[
                file("assets/scenes/a.ogt", "one", vec![3]),
                file("assets/scenes/b.ogt", "same", vec![])
            ]
        ));
        assert!(hand_over(
            ins,
            &[file("assets/scenes/a.ogt", "two", vec![3, 4])]
        ));
        assert_eq!(
            saved_of(ins),
            Some(
                serde_json::json!({ "scenes": [{ "source": "assets/scenes/a.ogt", "text": "two" }] })
            )
        );
        assert_eq!(saved_of(ins), None, "taken once");
        assert!(!hand_over(
            ins,
            &[file("assets/scenes/b.ogt", "same", vec![])]
        ));
        assert_eq!(saved_of(ins), None);
    }
}
