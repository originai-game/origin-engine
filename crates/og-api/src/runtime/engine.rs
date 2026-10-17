//! Declarations, engines, the tick, the dispatcher and the view tables.

use super::codec::{Layout, RecordKind};
use super::{ok, parse, refuse};
use og_core::reflect::Reflect;
use og_core::{Error, ErrorKind, Rng, Time};
use og_ecs::{Entity, FieldRef, QueryDesc, SchemaId, SystemContext, SystemDesc, World, WorldDesc};
use serde_json::{json, Value};
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

// ── host hooks ──────────────────────────────────────────────────────────────

/// One call into game code.
#[derive(Clone, Debug)]
pub struct Dispatch {
    /// 0 system, 1 command, 2 facts.
    pub kind: u32,
    pub engine: u32,
    /// The callback id the runtime's table holds the function under.
    pub id: u32,
    /// The system's query handle (systems), else 0.
    pub query: u32,
    pub tick: f64,
    pub dt: f64,
    pub elapsed: f64,
    /// Command arguments as a JSON list of strings; empty otherwise.
    pub payload: String,
}

/// One column handed to JavaScript: `len` bytes at `ptr` in engine memory.
#[derive(Clone, Copy, Debug)]
pub struct ColumnPtr {
    pub ptr: *mut u8,
    pub len: usize,
    pub writable: bool,
}

/// What a host provides to the runtime.
pub trait Hooks {
    /// Call the JavaScript dispatcher. `Ok` carries what it answered after
    /// a leading `=` (facts), `Err` the error it reported or threw.
    fn dispatch(&self, call: &Dispatch) -> Result<String, String>;
    /// The views handed out since the last call are retired. A host that
    /// copied columns out copies writable ones back now (the memory is still
    /// where it was). `moving` says whether column memory may move next (a
    /// structural change, a restore, a destroyed engine): then a host that
    /// aliased columns must detach what it handed out; otherwise it may keep
    /// a column's buffer for the next view of the same memory. Called after
    /// each system call, before each structural change outside systems, and
    /// at every tick boundary.
    fn invalidate_views(&self, moving: bool) {
        let _ = moving;
    }
    /// Wall-clock milliseconds, for the probe's timings only (never the tick).
    fn now_ms(&self) -> f64 {
        0.0
    }
}

/// A renderer the host attached to an engine (an OffscreenCanvas surface on
/// the web, an offscreen target on native).
pub trait RenderBackend {
    /// The IDL `RenderTier`: `webgpu`, `webgl2`, `metal`, `vulkan`.
    fn tier(&self) -> String;
    /// The render target in device pixels.
    fn size(&self) -> (u32, u32);
    /// Draw one frame of the scene.
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String>;
    /// Draw the scene offscreen and read it back; `None` where the backend
    /// cannot read back (the web, until og-render's web readback lands).
    fn capture(&mut self, scene: &og_render::RenderScene) -> Option<og_render::RgbaImage>;
}

/// Handles every engine on the thread hands out, tagged with their engine:
/// interface members (`ArchetypeView.column`, `RngStream.next`) carry only
/// their object's handle.
#[derive(Default)]
pub(crate) struct Handles {
    pub views: Table<(u32, ViewRecord)>,
    pub frame_queries: Table<(u32, QueryRec)>,
    pub world_queries: Table<(u32, QueryRec)>,
    pub streams: Table<(u32, Rng)>,
}

thread_local! {
    pub(crate) static HANDLES: RefCell<Handles> = RefCell::new(Handles::default());
    pub(crate) static HOOKS: RefCell<Option<Rc<dyn Hooks>>> = RefCell::new(None);
    pub(crate) static DECL: RefCell<Declarations> = RefCell::new(Declarations::default());
    static ENGINES: RefCell<Vec<Option<Rc<Engine>>>> = const { RefCell::new(Vec::new()) };
    static FRAME: Cell<*const Frame> = const { Cell::new(std::ptr::null()) };
    static QUEUED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn hooks() -> Option<Rc<dyn Hooks>> {
    HOOKS.with(|h| h.borrow().clone())
}

// ── declarations ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub kind: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct SystemDecl {
    pub name: String,
    pub reads: Vec<Token>,
    pub writes: Vec<Token>,
    pub without: Vec<String>,
    pub after: Vec<String>,
    pub before: Vec<String>,
    pub run: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct GameDecl {
    pub tick: u32,
    pub seed: u32,
    pub systems: Vec<u32>,
    pub resources: Vec<String>,
    pub commands: Vec<(String, u32)>,
    pub max_catch_up: u32,
    pub facts: Option<u32>,
    pub scene: String,
    pub scenes: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CommandDecl {
    pub handler: u32,
    // `og run --help` and the wiki list these (M5).
    #[allow(dead_code)]
    pub usage: String,
    #[allow(dead_code)]
    pub doc: String,
}

/// Everything the bundle declared, in call order.
#[derive(Default)]
pub(crate) struct Declarations {
    /// Game components and resources.
    pub records: Vec<Layout>,
    /// Declarations that were refused; an engine will not start with them.
    pub errors: Vec<String>,
    pub systems: Vec<SystemDecl>,
    pub games: Vec<GameDecl>,
    pub commands: BTreeMap<String, CommandDecl>,
}

// ── engines ─────────────────────────────────────────────────────────────────

/// A registered system as the engine runs it.
pub(crate) struct SystemEntry {
    pub name: String,
    pub callback: u32,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub columns: Vec<QueryColumn>,
    pub matched: u32,
    pub ms: f64,
}

#[derive(Clone)]
pub(crate) struct QueryColumn {
    pub component: String,
    pub field: FieldRef,
    pub writable: bool,
}

/// One archetype's view as handed out.
pub(crate) struct ViewRecord {
    pub count: u32,
    pub tick: u32,
    pub entities: *const Entity,
    pub columns: Vec<(String, ColumnPtr)>,
    pub types: Vec<String>,
}

pub(crate) enum QueryRec {
    /// A running system's archetypes: these view handles.
    Frame(Vec<u32>),
    /// `world.query()` outside systems.
    World(QueryDesc),
}

/// Handles valid from `base` to `base + items.len()`; clearing moves `base`
/// on, so an old handle never aliases a new one.
pub(crate) struct Table<T> {
    pub base: u32,
    pub items: Vec<T>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Table {
            base: 1,
            items: Vec::new(),
        }
    }
}

impl<T> Table<T> {
    pub fn push(&mut self, item: T) -> u32 {
        self.items.push(item);
        self.base + self.items.len() as u32 - 1
    }
    pub fn get(&self, handle: u32) -> Option<&T> {
        handle
            .checked_sub(self.base)
            .and_then(|i| self.items.get(i as usize))
    }
    pub fn clear(&mut self) {
        self.base = self.base.wrapping_add(self.items.len() as u32);
        self.items.clear();
    }
}

/// Input as the host collected it (keys and buttons by name).
#[derive(Default)]
pub(crate) struct InputState {
    pub queue: Vec<Value>,
    pub keys: BTreeSet<String>,
    pub pressed: BTreeSet<String>,
    pub released: BTreeSet<String>,
    pub buttons: BTreeSet<String>,
    pub buttons_pressed: BTreeSet<String>,
    pub pointer: (f32, f32),
    pub inside: bool,
    pub wheel: (f32, f32),
    /// Keys and buttons that were pressed and go up at the next tick.
    pub release_next: Vec<(bool, String)>,
}

pub(crate) struct State {
    pub layouts: BTreeMap<String, Layout>,
    /// Schema name by id index, for archetype type lists.
    pub names_by_id: Vec<String>,
    pub resources: Vec<String>,
    pub systems: Vec<SystemEntry>,
    pub commands: BTreeMap<String, CommandDecl>,
    pub names: BTreeMap<String, u64>,
    pub name_of: BTreeMap<u64, String>,
    pub name_order: Vec<String>,
    pub input: InputState,
    pub accepted: u32,
    pub paused: bool,
    pub seed: u32,
    pub rate: u32,
    pub max_catch_up: u32,
    pub accumulator_ms: f64,
    pub facts: Option<u32>,
    pub snapshots: Vec<(String, u32, Vec<u8>)>,
    pub scene: og_render::RenderScene,
    /// Materials `og.render.standardMaterial` made, by handle index.
    pub materials: Vec<og_render::MaterialHandle>,
    pub backend: Option<Box<dyn RenderBackend>>,
    pub step_ms: f64,
    pub render_ms: f64,
    pub warnings: Vec<String>,
}

pub(crate) struct Engine {
    pub id: u32,
    world: UnsafeCell<World>,
    pub stepping: Cell<bool>,
    pub state: RefCell<State>,
}

impl Engine {
    /// The world, outside a running system. The runtime never holds two of
    /// these at once and never across a dispatch.
    #[allow(clippy::mut_from_ref)]
    pub fn world(&self) -> Result<&mut World, String> {
        if self.stepping.get() {
            return Err("not available while a system runs: inside a system, only its own views, Time, Rng, alive, despawn, find and nameOf reach the world (og-ecs SystemContext has no packed-byte access yet)".into());
        }
        // SAFETY: single-threaded; `stepping` is false, so `World::step` does
        // not hold the world, and no caller keeps this borrow across a call
        // that could reach the world again.
        Ok(unsafe { &mut *self.world.get() })
    }
}

pub(crate) fn engine(id: u32) -> Result<Rc<Engine>, String> {
    ENGINES.with(|e| {
        e.borrow()
            .get(id as usize)
            .and_then(Clone::clone)
            .ok_or_else(|| format!("no engine {id}: it was destroyed, or never created"))
    })
}

/// The running system's frame, when one is running on `engine`.
pub(crate) struct Frame {
    pub engine: u32,
    pub system: usize,
    pub ctx: *mut (),
}

impl Frame {
    /// # Safety
    /// Only while the frame is published (its system's closure is blocked in
    /// the dispatch), and never held across another call into game code.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn ctx<'a>(&self) -> &'a mut SystemContext<'a> {
        &mut *(self.ctx as *mut SystemContext<'a>)
    }
}

/// The running system's frame on `engine`, if any.
pub(crate) fn running(engine: u32) -> Option<&'static Frame> {
    let p = FRAME.with(Cell::get);
    // SAFETY: the pointer is published only while `run_system` is on the stack.
    let f = unsafe { p.as_ref()? };
    (f.engine == engine).then_some(f)
}

/// Retires every view and system query on the thread (all engines: a view
/// is never worth keeping across another engine's tick either).
pub(crate) fn invalidate(moving: bool) {
    if let Some(h) = hooks() {
        h.invalidate_views(moving);
    }
    HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        t.views.clear();
        t.frame_queries.clear();
    });
}

/// Called before a structural change made outside systems.
pub(crate) fn before_structural(_eng: &Engine) {
    invalidate(true);
}

/// Set when the running system queued a structural change (a despawn): its
/// commands apply as it returns, so its views are retired as moving.
pub(crate) fn queued_structural() {
    QUEUED.with(|q| q.set(true));
}

fn clear_world_queries() {
    HANDLES.with(|t| t.borrow_mut().world_queries.clear());
}

// ── creation ────────────────────────────────────────────────────────────────

fn tokens(v: &Value) -> Vec<Token> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| {
                    Some(Token {
                        kind: t["kind"].as_str()?.to_string(),
                        name: t["name"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn declare_system(name: String, desc: &Value, run: u32) -> u32 {
    let decl = SystemDecl {
        name,
        reads: tokens(&desc["reads"]),
        writes: tokens(&desc["writes"]),
        without: tokens(&desc["without"])
            .into_iter()
            .map(|t| t.name)
            .collect(),
        after: strings(&desc["after"]),
        before: strings(&desc["before"]),
        run,
    };
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.systems.push(decl);
        d.systems.len() as u32 - 1
    })
}

pub(crate) fn declare_game(desc: &Value) -> u32 {
    let num = |k: &str, default: u32| desc[k].as_u64().map_or(default, |n| n as u32);
    let game = GameDecl {
        tick: num("tick", 60),
        seed: num("seed", 0),
        systems: desc["systems"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_u64().map(|n| n as u32))
                    .collect()
            })
            .unwrap_or_default(),
        resources: strings(&desc["resources"]),
        commands: desc["commands"]
            .as_object()
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_u64()? as u32)))
                    .collect()
            })
            .unwrap_or_default(),
        max_catch_up: num("maxCatchUp", 4),
        facts: desc["facts"].as_u64().map(|n| n as u32),
        scene: desc["scene"].as_str().unwrap_or("main").to_string(),
        scenes: desc["scenes"]
            .as_object()
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
    };
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.games.push(game);
        d.games.len() as u32 - 1
    })
}

/// Declaration order, then `after`/`before`: among the systems whose
/// predecessors have all been placed, the earliest declared goes next.
fn order(systems: &[SystemDecl]) -> Result<Vec<usize>, String> {
    let index: BTreeMap<&str, usize> = systems
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();
    if index.len() != systems.len() {
        return Err("two systems of the game share a name".into());
    }
    let mut preds: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); systems.len()];
    for (i, s) in systems.iter().enumerate() {
        for a in &s.after {
            let j = *index.get(a.as_str()).ok_or_else(|| {
                format!(
                    "system '{}' runs after '{a}', which the game does not list",
                    s.name
                )
            })?;
            preds[i].insert(j);
        }
        for b in &s.before {
            let j = *index.get(b.as_str()).ok_or_else(|| {
                format!(
                    "system '{}' runs before '{b}', which the game does not list",
                    s.name
                )
            })?;
            preds[j].insert(i);
        }
    }
    let mut placed = vec![false; systems.len()];
    let mut out = Vec::with_capacity(systems.len());
    while out.len() < systems.len() {
        let next = (0..systems.len())
            .find(|&i| !placed[i] && preds[i].iter().all(|&p| placed[p]))
            .ok_or_else(|| {
                let stuck: Vec<&str> = (0..systems.len())
                    .filter(|&i| !placed[i])
                    .map(|i| systems[i].name.as_str())
                    .collect();
                format!("the after/before constraints of {stuck:?} form a cycle")
            })?;
        placed[next] = true;
        out.push(next);
    }
    Ok(out)
}

fn err(e: Error) -> String {
    e.reason().to_string()
}

/// `og_engine_create`.
pub fn create(game: u32, options: &str) -> String {
    match create_inner(game, options) {
        Ok(id) => ok(json!(id)),
        Err(e) => refuse(e),
    }
}

fn create_inner(game: u32, options: &str) -> Result<u32, String> {
    let options = parse(Some(options))?;
    let (decl, records, errors, all_systems, commands) = DECL.with(|d| {
        let d = d.borrow();
        (
            d.games.get(game as usize).cloned(),
            d.records.clone(),
            d.errors.clone(),
            d.systems.clone(),
            d.commands.clone(),
        )
    });
    let game = decl.ok_or_else(|| format!("no game {game}: declare it with og.game first"))?;
    if !errors.is_empty() {
        return Err(format!(
            "the bundle has refused declarations: {}",
            errors.join("; ")
        ));
    }
    let seed = options["seed"].as_u64().map_or(game.seed, |s| s as u32);
    let mut world = World::new(&WorldDesc {
        tick_rate: game.tick,
        seed,
    })
    .map_err(err)?;

    let mut state = State {
        layouts: BTreeMap::new(),
        names_by_id: Vec::new(),
        resources: vec![Time::NAME.to_string(), Rng::NAME.to_string()],
        systems: Vec::new(),
        commands: BTreeMap::new(),
        names: BTreeMap::new(),
        name_of: BTreeMap::new(),
        name_order: Vec::new(),
        input: InputState::default(),
        accepted: 0,
        paused: false,
        seed,
        rate: game.tick,
        max_catch_up: game.max_catch_up.max(1),
        accumulator_ms: 0.0,
        facts: game.facts,
        snapshots: Vec::new(),
        scene: og_render::RenderScene::new(),
        materials: Vec::new(),
        backend: None,
        step_ms: 0.0,
        render_ms: 0.0,
        warnings: Vec::new(),
    };

    // Engine-owned records in IDL order. A record og-core cannot store yet
    // is left out with a warning; using its token is then refused by name.
    for decl in crate::host::RECORDS {
        if decl.schema == Time::NAME || decl.schema == Rng::NAME {
            continue;
        }
        match Layout::from_decl(decl) {
            Ok(layout) => {
                world.register_schema(layout.schema()).map_err(err)?;
                if layout.kind == RecordKind::Resource {
                    let bytes = layout.encode(None, None)?;
                    let id = world.id_by_name(&layout.schema).map_err(err)?;
                    world.insert_resource_bytes(id, &bytes).map_err(err)?;
                    state.resources.push(layout.schema.clone());
                }
                state.layouts.insert(layout.schema.clone(), layout);
            }
            Err(e) => state
                .warnings
                .push(format!("{} not registered: {e}", decl.schema)),
        }
    }
    // The game's own records, in declaration order; its resources start at
    // their defaults.
    for layout in &records {
        world.register_schema(layout.schema()).map_err(err)?;
        if layout.kind == RecordKind::Resource {
            let bytes = layout.encode(None, None)?;
            let id = world.id_by_name(&layout.schema).map_err(err)?;
            world.insert_resource_bytes(id, &bytes).map_err(err)?;
            state.resources.push(layout.schema.clone());
        }
        state.layouts.insert(layout.schema.clone(), layout.clone());
    }
    for name in &game.resources {
        if !state.resources.contains(name) {
            return Err(format!(
                "the game lists resource '{name}', which was never declared with defineResource"
            ));
        }
    }

    // Systems: the game's, ordered.
    let mut systems = Vec::new();
    for &s in &game.systems {
        systems.push(
            all_systems
                .get(s as usize)
                .cloned()
                .ok_or_else(|| format!("the game lists system {s}, which was never declared"))?,
        );
    }
    let engine_id = ENGINES.with(|e| e.borrow().len() as u32);
    for i in order(&systems)? {
        let s = &systems[i];
        let is_component = |t: &Token| t.kind == "component";
        let mut q = QueryDesc::new();
        let mut columns = Vec::new();
        for t in s.writes.iter().filter(|t| is_component(t)) {
            q.write.push(t.name.clone());
        }
        for t in s.reads.iter().filter(|t| is_component(t)) {
            if !q.write.contains(&t.name) && !q.read.contains(&t.name) {
                q.read.push(t.name.clone());
            }
        }
        q.without = s.without.clone();
        for (names, writable) in [(&q.write, true), (&q.read, false)] {
            for name in names {
                let layout = state.layouts.get(name).ok_or_else(|| {
                    format!(
                        "system '{}' declares '{name}', which is not a registered component",
                        s.name
                    )
                })?;
                let id = world.id_by_name(name).map_err(err)?;
                for f in &layout.fields {
                    columns.push(QueryColumn {
                        component: name.clone(),
                        field: world.field_by_id(id, &f.name).map_err(err)?,
                        writable,
                    });
                }
            }
        }
        let names = |ts: &[Token]| ts.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        let desc = SystemDesc {
            name: s.name.clone(),
            reads: names(&s.reads),
            writes: names(&s.writes),
            queries: vec![q],
        };
        let index = state.systems.len();
        world
            .add_system(desc, move |ctx| run_system(engine_id, index, ctx))
            .map_err(err)?;
        state.systems.push(SystemEntry {
            name: s.name.clone(),
            callback: s.run,
            reads: names(&s.reads),
            writes: names(&s.writes),
            columns,
            matched: 0,
            ms: 0.0,
        });
    }
    state.commands = commands;
    for (name, handler) in &game.commands {
        state.commands.insert(
            name.clone(),
            CommandDecl {
                handler: *handler,
                usage: String::new(),
                doc: String::new(),
            },
        );
    }
    if let Some(path) = game.scenes.get(&game.scene) {
        state.warnings.push(format!(
            "scene '{}' ({path}) was not loaded: scene loading lands with the cook (M4); the world starts empty",
            game.scene
        ));
    }
    state.names_by_id = world.schemas().into_iter().map(|s| s.name).collect();
    let engine = Rc::new(Engine {
        id: engine_id,
        world: UnsafeCell::new(world),
        stepping: Cell::new(false),
        state: RefCell::new(state),
    });
    ENGINES.with(|e| e.borrow_mut().push(Some(engine)));
    Ok(engine_id)
}

/// `og_engine_destroy`.
pub fn destroy(id: u32) -> bool {
    let Ok(eng) = engine(id) else { return false };
    if eng.stepping.get() {
        return false;
    }
    before_structural(&eng);
    ENGINES.with(|e| {
        if let Some(slot) = e.borrow_mut().get_mut(id as usize) {
            *slot = None;
        }
    });
    true
}

/// Attach (or detach, with `None`) a renderer to an engine.
pub fn set_render_backend(id: u32, backend: Option<Box<dyn RenderBackend>>) -> Result<(), String> {
    let eng = engine(id)?;
    eng.state.borrow_mut().backend = backend;
    Ok(())
}

// ── the tick ────────────────────────────────────────────────────────────────

/// Applies queued input to the host's input state and the Input resource,
/// at a tick boundary.
fn apply_input(eng: &Engine) -> Result<(), String> {
    let mut st = eng.state.borrow_mut();
    let input = &mut st.input;
    input.pressed.clear();
    input.released.clear();
    input.buttons_pressed.clear();
    input.wheel = (0.0, 0.0);
    for (button, key) in std::mem::take(&mut input.release_next) {
        if button {
            input.buttons.remove(&key);
        } else if input.keys.remove(&key) {
            input.released.insert(key);
        }
    }
    for ev in std::mem::take(&mut input.queue) {
        let action = ev["action"].as_str().unwrap_or_default().to_string();
        match ev["kind"].as_str() {
            Some("key") => {
                let key = ev["key"].as_str().unwrap_or_default().to_string();
                match action.as_str() {
                    "down" | "press" => {
                        if input.keys.insert(key.clone()) {
                            input.pressed.insert(key.clone());
                        }
                        if action == "press" {
                            input.release_next.push((false, key));
                        }
                    }
                    _ => {
                        if input.keys.remove(&key) {
                            input.released.insert(key);
                        }
                    }
                }
            }
            Some("pointer") => {
                let x = ev["x"].as_f64().unwrap_or(0.0) as f32;
                let y = ev["y"].as_f64().unwrap_or(0.0) as f32;
                input.pointer = (x, y);
                input.inside = true;
                let button = ev["button"].as_str().unwrap_or("none").to_string();
                match action.as_str() {
                    "down" | "click" if button != "none" => {
                        if input.buttons.insert(button.clone()) {
                            input.buttons_pressed.insert(button.clone());
                        }
                        if action == "click" {
                            input.release_next.push((true, button));
                        }
                    }
                    "up" => {
                        input.buttons.remove(&button);
                    }
                    _ => {}
                }
            }
            Some("wheel") => {
                input.pointer = (
                    ev["x"].as_f64().unwrap_or(0.0) as f32,
                    ev["y"].as_f64().unwrap_or(0.0) as f32,
                );
                input.wheel.0 += ev["deltaX"].as_f64().unwrap_or(0.0) as f32;
                input.wheel.1 += ev["deltaY"].as_f64().unwrap_or(0.0) as f32;
            }
            _ => {}
        }
    }
    let value = json!({
        "pointer": { "x": input.pointer.0, "y": input.pointer.1 },
        "pointerInside": input.inside,
        "wheel": { "x": input.wheel.0, "y": input.wheel.1 },
    });
    let Some(layout) = st.layouts.get("og.Input") else {
        return Ok(());
    };
    let bytes = layout.encode(None, Some(&value))?;
    drop(st);
    let world = eng.world()?;
    let id = world.id_by_name("og.Input").map_err(err)?;
    world.insert_resource_bytes(id, &bytes).map_err(err)
}

/// One fixed step. `Err` names the tick and the failing system.
pub(crate) fn step(eng: &Engine) -> Result<(), String> {
    apply_input(eng)?;
    invalidate(false);
    clear_world_queries();
    let t0 = hooks().map_or(0.0, |h| h.now_ms());
    eng.stepping.set(true);
    // SAFETY: the one place the world is borrowed for a step; re-entrant
    // calls see `stepping` and go through the running system's frame.
    let result = unsafe { &mut *eng.world.get() }.step();
    eng.stepping.set(false);
    invalidate(false);
    let mut st = eng.state.borrow_mut();
    st.step_ms = hooks().map_or(0.0, |h| h.now_ms()) - t0;
    result.map_err(err)
}

/// `og_engine_advance`.
pub fn advance(id: u32, steps: u32) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    if eng.stepping.get() {
        return refuse("advance was called from inside a system");
    }
    for _ in 0..steps {
        if let Err(e) = step(&eng) {
            return refuse(e);
        }
    }
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    ok(json!(tick))
}

/// `og_engine_frame`.
pub fn frame(id: u32, elapsed_ms: f64) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let ticks = {
        let mut st = eng.state.borrow_mut();
        if st.paused || !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
            0
        } else {
            let tick_ms = 1000.0 / f64::from(st.rate);
            let cap = f64::from(st.max_catch_up) * tick_ms;
            st.accumulator_ms = (st.accumulator_ms + elapsed_ms).min(cap);
            let n = (st.accumulator_ms / tick_ms).floor() as u32;
            st.accumulator_ms -= f64::from(n) * tick_ms;
            n
        }
    };
    for _ in 0..ticks {
        if let Err(e) = step(&eng) {
            return refuse(e);
        }
    }
    let rendered = match render(&eng) {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    json!({ "ok": true, "ticks": ticks, "tick": tick, "rendered": rendered }).to_string()
}

/// The render scene of the current tick: the camera from the Camera resource.
/// Meshes join once og.Mesh can register (its bool fields need FieldType::Bool).
pub(crate) fn extract_scene(eng: &Engine) -> Result<(), String> {
    let camera = super::api::resource_value(eng, "og.Camera").ok();
    let mut st = eng.state.borrow_mut();
    st.scene.instances.clear();
    if let Some(c) = camera {
        let v3 = |v: &Value| {
            glam::Vec3::new(
                v["x"].as_f64().unwrap_or(0.0) as f32,
                v["y"].as_f64().unwrap_or(0.0) as f32,
                v["z"].as_f64().unwrap_or(0.0) as f32,
            )
        };
        let q = &c["quaternion"];
        let rot = glam::Quat::from_xyzw(
            q["x"].as_f64().unwrap_or(0.0) as f32,
            q["y"].as_f64().unwrap_or(0.0) as f32,
            q["z"].as_f64().unwrap_or(0.0) as f32,
            q["w"].as_f64().unwrap_or(1.0) as f32,
        );
        let position = v3(&c["position"]);
        let bg = &c["background"];
        // Position, aim and clip planes only: og-render's projection and
        // exposure types are being reworked in M3, so fov, zoom and exposure
        // reach the renderer once that API is committed.
        let cam = &mut st.scene.camera;
        cam.position = position;
        cam.target = position + rot * glam::Vec3::NEG_Z;
        cam.up = rot * glam::Vec3::Y;
        cam.near = c["near"].as_f64().unwrap_or(0.1) as f32;
        cam.far = c["far"].as_f64().unwrap_or(2000.0) as f32;
        st.scene.environment.background = og_render::Background::Color(glam::Vec3::new(
            bg["r"].as_f64().unwrap_or(0.0) as f32,
            bg["g"].as_f64().unwrap_or(0.0) as f32,
            bg["b"].as_f64().unwrap_or(0.0) as f32,
        ));
    }
    Ok(())
}

fn render(eng: &Engine) -> Result<bool, String> {
    if eng.state.borrow().backend.is_none() {
        return Ok(false);
    }
    extract_scene(eng)?;
    let t0 = hooks().map_or(0.0, |h| h.now_ms());
    let mut st = eng.state.borrow_mut();
    let st = &mut *st;
    if let Some(b) = st.backend.as_mut() {
        b.render(&st.scene)?;
    }
    st.render_ms = hooks().map_or(0.0, |h| h.now_ms()) - t0;
    Ok(true)
}

// ── game code calls ─────────────────────────────────────────────────────────

/// Runs one registered system: borrow its columns, publish them as views,
/// call the dispatcher, then retire the views.
fn run_system(engine_id: u32, index: usize, ctx: &mut SystemContext<'_>) -> og_core::Result<()> {
    let fail = |m: String| Error::new(ErrorKind::Invalid, m);
    let eng = engine(engine_id).map_err(fail)?;
    let (callback, columns) = {
        let st = eng.state.borrow();
        let s = &st.systems[index];
        (s.callback, s.columns.clone())
    };
    let time = ctx.time();
    let tick = time.tick as u32;
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    let mut records = Vec::new();
    for a in ctx.query(0)? {
        let mut cols = Vec::with_capacity(columns.len());
        for c in &columns {
            let ptr = if c.writable {
                let mut b = a.write_bytes(c.field)?;
                let p = ColumnPtr {
                    ptr: b.as_mut_ptr(),
                    len: b.len(),
                    writable: true,
                };
                writes.push(b);
                p
            } else {
                let b = a.read_bytes(c.field)?;
                let p = ColumnPtr {
                    ptr: b.as_ptr() as *mut u8,
                    len: b.len(),
                    writable: false,
                };
                reads.push(b);
                p
            };
            cols.push((c.component.clone(), ptr));
        }
        records.push(ViewRecord {
            count: a.len() as u32,
            tick,
            entities: a.entities().as_ptr(),
            columns: cols,
            types: Vec::new(),
        });
        let st = eng.state.borrow();
        let types = a
            .types()
            .iter()
            .filter_map(|id| st.names_by_id.get(id.index() as usize).cloned())
            .collect();
        records.last_mut().expect("just pushed").types = types;
    }
    let matched: u32 = records.iter().map(|r| r.count).sum();
    let query = HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let handles = records
            .into_iter()
            .map(|r| t.views.push((engine_id, r)))
            .collect();
        t.frame_queries.push((engine_id, QueryRec::Frame(handles)))
    });
    let frame = Frame {
        engine: engine_id,
        system: index,
        ctx: ctx as *mut SystemContext<'_> as *mut (),
    };
    let prev = FRAME.with(|f| f.replace(&frame));
    let hooks = hooks();
    let t0 = hooks.as_ref().map_or(0.0, |h| h.now_ms());
    let result = match &hooks {
        Some(h) => h.dispatch(&Dispatch {
            kind: 0,
            engine: engine_id,
            id: callback,
            query,
            tick: tick as f64,
            dt: f64::from(time.dt),
            elapsed: time.elapsed,
            payload: String::new(),
        }),
        None => Err("no dispatcher is installed (og_host_set_dispatcher)".into()),
    };
    FRAME.with(|f| f.set(prev));
    invalidate(QUEUED.with(|q| q.replace(false)));
    {
        let mut st = eng.state.borrow_mut();
        let ms = hooks.as_ref().map_or(0.0, |h| h.now_ms()) - t0;
        let s = &mut st.systems[index];
        s.matched = matched;
        s.ms = ms;
    }
    drop(writes);
    drop(reads);
    result.map(|_| ()).map_err(fail)
}

/// Calls a command handler (or the facts function) between ticks.
pub(crate) fn call(eng: &Engine, kind: u32, id: u32, payload: String) -> Result<String, String> {
    let hooks = hooks().ok_or("no dispatcher is installed (og_host_set_dispatcher)")?;
    let time = eng.world()?.time();
    let result = hooks.dispatch(&Dispatch {
        kind,
        engine: eng.id,
        id,
        query: 0,
        tick: time.tick as f64,
        dt: f64::from(time.dt),
        elapsed: time.elapsed,
        payload,
    });
    invalidate(false);
    clear_world_queries();
    result
}

/// `og_engine_command`.
pub fn command(id: u32, action: &str, args: &str) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    let handler = eng.state.borrow().commands.get(action).map(|c| c.handler);
    let Some(handler) = handler else {
        let known: Vec<String> = eng.state.borrow().commands.keys().cloned().collect();
        return refuse(format!(
            "no command action '{action}'; the game registers {known:?}"
        ));
    };
    let args = match parse(Some(args)) {
        Ok(Value::Array(a)) => Value::Array(a),
        Ok(Value::Null) => json!([]),
        _ => return refuse("command arguments are a JSON list of strings"),
    };
    match call(&eng, 1, handler, args.to_string()) {
        Ok(_) => super::ok_void(),
        Err(e) => refuse(format!("command '{action}': {e}")),
    }
}

/// `og_engine_write_resource`.
pub fn write_resource(world: u32, resource: &str, value: &str) -> String {
    let result = (|| {
        let eng = engine(world)?;
        let value = parse(Some(value))?;
        let st = eng.state.borrow();
        let layout = st
            .layouts
            .get(resource)
            .filter(|l| l.kind == RecordKind::Resource)
            .ok_or_else(|| format!("'{resource}' is not a registered resource"))?
            .clone();
        drop(st);
        if let Some(f) = layout
            .fields
            .iter()
            .find(|f| f.readonly && value.get(&f.name).is_some())
        {
            return Err(format!("{resource}.{} is read only", f.name));
        }
        let w = eng.world()?;
        let id = w.id_by_name(resource).map_err(err)?;
        let current = w.resource_bytes(id).map_err(err)?;
        let bytes = layout.encode(Some(&current), Some(&value))?;
        w.insert_resource_bytes(id, &bytes).map_err(err)
    })();
    match result {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

// ── views ───────────────────────────────────────────────────────────────────

/// The columns of `component` in view `view`, in field order, or why not.
pub fn view_columns(view: u32, component: &str) -> Result<Vec<ColumnPtr>, String> {
    HANDLES.with(|t| {
        let t = t.borrow();
        let (_, rec) = t.views.get(view).ok_or(
            "stale view: views last for the system call (or, outside systems, until the next structural change) they were handed out in",
        )?;
        let cols: Vec<ColumnPtr> = rec
            .columns
            .iter()
            .filter(|(c, _)| c == component)
            .map(|(_, p)| *p)
            .collect();
        if cols.is_empty() {
            return Err(if rec.types.iter().any(|t| t == component) {
                format!("'{component}' is not among the reads or writes this view was made with")
            } else {
                format!("this archetype has no '{component}'")
            });
        }
        Ok(cols)
    })
}

/// The entities of view `view` as (index, generation) pairs; empty for a
/// stale view.
pub fn view_entities(view: u32) -> Vec<u32> {
    HANDLES.with(|t| {
        let t = t.borrow();
        let Some((_, rec)) = t.views.get(view) else {
            return Vec::new();
        };
        // SAFETY: the record is live, so the archetype's entity list has not
        // moved since the view was made.
        let ents = unsafe { std::slice::from_raw_parts(rec.entities, rec.count as usize) };
        let mut out = Vec::with_capacity(ents.len() * 2);
        for e in ents {
            out.push(e.index());
            out.push(e.generation());
        }
        out
    })
}

/// Builds view records for a world query outside systems.
pub(crate) fn world_query_views(eng: &Engine, desc: &QueryDesc) -> Result<Vec<u32>, String> {
    let world = eng.world()?;
    let tick = world.time().tick as u32;
    let st = eng.state.borrow();
    let mut out = Vec::new();
    for a in world.query(desc).map_err(err)? {
        let mut cols = Vec::new();
        for name in desc.write.iter() {
            let layout = st
                .layouts
                .get(name)
                .ok_or_else(|| format!("'{name}' is not a registered component"))?;
            let id = world.id_by_name(name).map_err(err)?;
            for f in &layout.fields {
                let field = world.field_by_id(id, &f.name).map_err(err)?;
                let mut b = a.write_bytes(field).map_err(err)?;
                cols.push((
                    name.clone(),
                    ColumnPtr {
                        ptr: b.as_mut_ptr(),
                        len: b.len(),
                        writable: true,
                    },
                ));
            }
        }
        let types = a
            .types()
            .iter()
            .filter_map(|id: &SchemaId| st.names_by_id.get(id.index() as usize).cloned())
            .collect();
        let rec = ViewRecord {
            count: a.len() as u32,
            tick,
            entities: a.entities().as_ptr(),
            columns: cols,
            types,
        };
        out.push(HANDLES.with(|t| t.borrow_mut().views.push((eng.id, rec))));
    }
    Ok(out)
}
