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
//! - **Not world state.** Nothing it holds is in a snapshot or the hash,
//!   and a publish build has none of it (the module, its action and its
//!   key are compiled out; `inspect` is then an unknown action).

mod model;
mod panel;

pub(crate) use model::Edit;

use crate::engine::Engine;
use crate::extract::RenderState;
use og_ecs::{Entity, World};
use og_input::{InjectedInput, KeyAction, PointerAction};
use std::collections::{BTreeMap, BTreeSet};

/// The overlay layer the inspector draws from: above the game's UI and
/// every screen layer lifted past it (og_render::screen_layer).
const LAYER: i32 = 1 << 30;

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

/// What the panel asked for this frame, done after it is built.
pub(crate) enum Request {
    /// A `set`: the target, its command text and the edit to remember
    /// (`merge`: one more step of the same drag, folded into the last).
    Tune {
        edit: Edit,
        merge: bool,
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
            action, x, y, id, ..
        } => {
            if *id != 0 {
                return false;
            }
            let over = ins.over_panel(*x, *y);
            let mut take = over || ins.captured;
            match action {
                PointerAction::Down => {
                    if over {
                        ins.captured = true;
                    } else if ins.picking {
                        ins.pick_at = Some((*x, *y));
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
            let take = ins.editing || ins.over_panel(pointer.x, pointer.y);
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
    let requests = {
        let Ok(mut rs) = eng.render.try_borrow_mut() else {
            return;
        };
        let fonts = rs.ui.as_ref().map(|u| {
            let u = u.lock();
            (u.fonts().clone(), u.icons().clone())
        });
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
            },
        );
        if let Some(id) = ins.pending_select.take() {
            ins.selected = probe.names().resolve(world, &id).or(ins.selected);
            ins.tab = 0;
        }
        std::mem::take(&mut ins.requests)
    };
    for r in requests {
        apply(eng, r);
    }
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
        Request::Save => {
            let saved = crate::probe::with_probe(eng, false, |p, w| p.save_scene(w, true, false));
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
                        format!("{lines} lines ready; this host writes no files")
                    };
                    if skipped > 0 {
                        msg.push_str(&format!(" ({skipped} not in scene text)"));
                    }
                    if let Ok(mut rs) = eng.render.try_borrow_mut()
                        && let Some(ins) = rs.inspector.as_mut()
                    {
                        if written > 0 {
                            ins.dirty.clear();
                        }
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
            }
        }
    }
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

/// Draws the panel over the frame (the last UI frame it built).
pub(crate) fn emit(rs: &mut RenderState) {
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
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w.max(1.0), h.max(1.0), 0.0));
    let first = overlay.texts.len();
    frame.emit(&og_ui::EmitDesc { layer: LAYER, time }, overlay);
    // A multi-page font draws once per page with one layout.
    let mut more = Vec::new();
    for t in &overlay.texts[first..] {
        for page in rs.ui_pages.get(&t.font).into_iter().flatten().skip(1) {
            more.push(og_render::Text {
                font: *page,
                ..t.clone()
            });
        }
    }
    overlay.texts.extend(more);
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
                "dirty": ins.dirty.iter().collect::<Vec<_>>(),
                "changedByGame": ins.flash.keys().collect::<Vec<_>>(),
                "status": ins.status,
                "elements": elements,
            }))
        },
    );
    let _ = probe.register_command(
        "inspect",
        og_probe::CommandDesc {
            usage: "inspect[:on|off|<entity>|key:<code>]".into(),
            doc: "the inspector (development builds): open it (on, or no argument), close it (off), open it on an entity (its name or #<index>.<generation>), or set the key that toggles it (key:F2; F1 by default, with Ctrl when the game binds it)".into(),
        },
        move |world, args| {
            let eng = crate::engine::engine(engine_id)?;
            let arg = args.first().map(String::as_str).unwrap_or("on");
            let selected = match arg {
                "on" | "off" => None,
                "key" => None,
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
                "key" => {
                    let code = args.get(1).map(String::as_str).unwrap_or("");
                    if !og_input::KEY_CODES.contains(&code) {
                        return Err(format!(
                            "inspect:key:{code}: not a KeyboardEvent.code (F1, F2, Backquote)"
                        ));
                    }
                    ins.key = code.to_string();
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
