//! Declarations, engines, the tick, the dispatcher and the view tables.

use super::assets::{Assets, Fetch, Status};
use super::codec::{Layout, RecordKind};
use super::extract::{DrawLayouts, RenderState};
use super::probe::{self as rprobe, NameOp};
use super::{ok, parse, refuse};
use og_core::reflect::Reflect;
use og_core::{Error, ErrorKind, Rng, Time};
use og_ecs::{Entity, FieldRef, QueryDesc, SchemaId, SystemContext, SystemDesc, World, WorldDesc};
use serde_json::{Value, json};
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
    /// The bytes of a cooked file (`<root>/cooked/...`): read now (native),
    /// answered from what was fetched (web), or `Pending` while it is fetched.
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: this host reads no files"))
    }
    /// A cooked texture as RGBA8 for the renderer: og-cook's transcoder
    /// (native, feature `transcode`) by default; the web host transcodes with
    /// Binomial's Basis transcoder.
    fn texture(&self, texture: &og_cook::runtime::CookedTexture) -> Fetch<og_render::TextureData> {
        match texture.to_texture_data() {
            Ok(t) => Fetch::Ready(t),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

pub use super::extract::RenderBackend;

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
    // Never dropped by the thread-local destructor: at process exit that
    // destructor would tear down every engine's renderer, and wgpu's buffer
    // drop reads another thread-local (its lock trace under debug
    // assertions) that is already gone, which aborts the process. The
    // process is ending, so the engines are leaked on purpose; an explicit
    // destroy still drops them.
    static ENGINES: std::mem::ManuallyDrop<RefCell<Vec<Option<Rc<Engine>>>>> =
        const { std::mem::ManuallyDrop::new(RefCell::new(Vec::new())) };
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
    /// GameDesc.render (RenderOptions), as given.
    pub render: Value,
    /// GameDesc.start, called once before the first tick.
    pub start: Option<u32>,
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

pub(crate) struct State {
    pub layouts: BTreeMap<String, Layout>,
    /// Schema name by id index, for archetype type lists.
    pub names_by_id: Vec<String>,
    pub resources: Vec<String>,
    pub systems: Vec<SystemEntry>,
    pub commands: BTreeMap<String, CommandDecl>,
    pub names: BTreeMap<String, u64>,
    pub name_of: BTreeMap<u64, String>,
    /// Name changes the probe has not seen yet.
    pub name_ops: Vec<NameOp>,
    pub rate: u32,
    pub max_catch_up: u32,
    pub accumulator_ms: f64,
    pub facts: Option<u32>,
    /// The scene GameDesc.scene names, while its bundle is still loading.
    pub boot_scene: Option<String>,
    /// GameDesc.start, until it has run.
    pub start: Option<u32>,
    pub step_ms: f64,
    pub render_ms: f64,
    pub warnings: Vec<String>,
    /// Log lines for `og_engine_take_log`.
    pub log: Vec<Value>,
    /// Command outputs for `og_engine_take_outputs`.
    pub outputs: Vec<og_probe::Output>,
    /// A rebuilt bundle's state, waiting for the probe's reload to swap it in.
    pub reloaded: Option<Box<Assembled>>,
    /// The physics profile (2 or 3), from the host's options.
    pub profile: u32,
    /// The engine modules' handles: physics (meshes, queries) and the UI.
    pub physics: Option<og_physics::PhysicsSystem>,
    pub ui: Option<og_ui::SharedUi>,
    /// Every mesh registered with physics, replayed into a reload's physics.
    pub physics_meshes: BTreeMap<u64, og_physics::MeshDesc>,
    /// UI warnings already logged, so a repeated one is logged once.
    pub ui_warned: BTreeSet<String>,
    /// og-anim on this world, and its assets (replayed into a reload's world).
    pub anim: Option<og_anim::AnimationSystem>,
    pub anim_assets: super::anim::AnimAssets,
    /// Tweens, flipbooks, tilemaps, camera feel and sprites on this world,
    /// and their assets (replayed into a reload's world).
    pub feel: Option<super::feel::Feel>,
    pub feel_assets: super::feel::FeelAssets,
}

pub(crate) struct Engine {
    pub id: u32,
    world: UnsafeCell<World>,
    pub stepping: Cell<bool>,
    pub state: RefCell<State>,
    /// og-probe over this engine's world: the command channel and protocol 5.
    pub probe: RefCell<og_probe::Probe>,
    /// The renderer side, shared with the probe's capture hook.
    pub render: Rc<RefCell<RenderState>>,
    pub assets: RefCell<Assets>,
}

impl Engine {
    /// The world, outside a running system. The runtime never holds two of
    /// these at once and never across a dispatch.
    #[allow(clippy::mut_from_ref)]
    pub fn world(&self) -> Result<&mut World, String> {
        if self.stepping.get() {
            return Err("not available while a system runs: inside a system, only its own views, Time, Rng, Input, alive, despawn, find and nameOf reach the world".into());
        }
        // SAFETY: single-threaded; `stepping` is false, so `World::step` does
        // not hold the world, and no caller keeps this borrow across a call
        // that could reach the world again.
        Ok(unsafe { &mut *self.world.get() })
    }
}

/// Every live engine's id.
pub(crate) fn engine_ids() -> Vec<u32> {
    ENGINES.with(|e| {
        e.borrow()
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_some())
            .map(|(i, _)| i as u32)
            .collect()
    })
}

/// `og_host_begin_bundle`: the next bundle's component, resource and command
/// declarations replace the previous bundle's; systems and games keep their
/// handles (a hot reload imports the new bundle after this).
pub fn begin_bundle() {
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.records.clear();
        d.errors.clear();
        d.commands.clear();
    });
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
        unsafe { &mut *(self.ctx as *mut SystemContext<'a>) }
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

pub(crate) fn clear_world_queries() {
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
        render: desc["render"].clone(),
        start: desc["start"].as_u64().map(|n| n as u32),
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

/// A world assembled from a game's declarations, with the runtime's view of
/// it. Engine creation and hot reload both build one.
pub(crate) struct Assembled {
    pub world: World,
    pub probe: Option<og_probe::Probe>,
    pub layouts: BTreeMap<String, Layout>,
    pub names_by_id: Vec<String>,
    pub resources: Vec<String>,
    pub systems: Vec<SystemEntry>,
    pub commands: BTreeMap<String, CommandDecl>,
    pub warnings: Vec<String>,
    pub game: GameDecl,
    /// Physics, the UI and the audio link of the new world.
    pub mounted: super::modules::Mounted,
    /// og-anim on the new world.
    pub anim: og_anim::AnimationSystem,
    /// Tweens, flipbooks, tilemaps, camera feel and sprites of the new world.
    pub feel: super::feel::Feel,
    pub feel_records: super::feel::Records,
}

/// Engine-owned records og-probe or og-input register themselves.
const REGISTERED_BY_MODULES: &[&str] = &[
    "og.Input",
    "og.KeyDown",
    "og.KeyUp",
    "og.PointerDown",
    "og.PointerUp",
    "og.PointerMove",
    "og.Wheel",
    "og.Pointers",
    "og.Gamepad",
];

/// Builds a world from game `game` and every declaration so far: the
/// engine's records (IDL order), the game's records (declaration order), the
/// probe (which registers `og.Input`), then the systems (declared order with
/// `after`/`before`).
pub(crate) fn assemble(
    game: u32,
    seed: Option<u32>,
    engine_id: u32,
    profile: u32,
    previous: Option<(&og_ui::SharedUi, og_audio::AudioLink)>,
) -> Result<Assembled, String> {
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
    let seed = seed.unwrap_or(game.seed);
    let mut world = World::new(&WorldDesc {
        tick_rate: game.tick,
        seed,
    })
    .map_err(err)?;
    let mut layouts = BTreeMap::new();
    let mut resources = vec![Time::NAME.to_string(), Rng::NAME.to_string()];
    let mut warnings = Vec::new();

    // Engine-owned records in IDL order. A record og-core cannot store yet
    // is left out with a warning; using its token is then refused by name.
    for decl in og_api::host::RECORDS {
        if decl.schema == Time::NAME
            || decl.schema == Rng::NAME
            || REGISTERED_BY_MODULES.contains(&decl.schema)
            || super::anim::REGISTERED_BY_ANIM.contains(&decl.schema)
            || super::feel::REGISTERED_BY_FEEL.contains(&decl.schema)
        {
            continue;
        }
        match Layout::from_decl(decl).and_then(|l| {
            world.register_schema(l.schema()).map_err(err)?;
            Ok(l)
        }) {
            Ok(layout) => {
                if layout.kind == RecordKind::Resource {
                    let bytes = layout.encode(None, None)?;
                    let id = world.id_by_name(&layout.schema).map_err(err)?;
                    world.insert_resource_bytes(id, &bytes).map_err(err)?;
                    resources.push(layout.schema.clone());
                }
                layouts.insert(layout.schema.clone(), layout);
            }
            Err(e) => warnings.push(format!("{} not registered: {e}", decl.schema)),
        }
    }
    // og-anim's events (their names are str64), before any system can read them.
    for layout in super::anim::register_records(&mut world)? {
        layouts.insert(layout.schema.clone(), layout);
    }
    {
        world.register::<og_anim::FlipbookEvent>().map_err(err)?;
        let id = world.id_by_name("og.FlipbookEvent").map_err(err)?;
        let layout = Layout::from_schema(world.schema(id).map_err(err)?, RecordKind::Event);
        layouts.insert(layout.schema.clone(), layout);
    }
    // The game's own records, in declaration order; its resources start at
    // their defaults.
    for layout in &records {
        world.register_schema(layout.schema()).map_err(err)?;
        if layout.kind == RecordKind::Resource {
            let bytes = layout.encode(None, None)?;
            let id = world.id_by_name(&layout.schema).map_err(err)?;
            world.insert_resource_bytes(id, &bytes).map_err(err)?;
            resources.push(layout.schema.clone());
        }
        layouts.insert(layout.schema.clone(), layout.clone());
    }
    for name in &game.resources {
        if !resources.contains(name) {
            return Err(format!(
                "the game lists resource '{name}', which was never declared with defineResource"
            ));
        }
    }
    // The probe, before any system that reads og.Input and before any snapshot.
    let probe = og_probe::Probe::new(&mut world)
        .map_err(|e| format!("the probe could not attach: {}", e.reason()))?;
    resources.push("og.Input".to_string());
    resources.push("og.Pointers".to_string());
    resources.push("og.Gamepad".to_string());
    // The input events and og-input's further resources are read through
    // the layout og-input registered.
    for decl in og_api::host::RECORDS {
        if !REGISTERED_BY_MODULES.contains(&decl.schema) || decl.schema == "og.Input" {
            continue;
        }
        let kind = match decl.kind {
            "event" => RecordKind::Event,
            "resource" => RecordKind::Resource,
            _ => continue,
        };
        let id = world.id_by_name(decl.schema).map_err(err)?;
        let layout = Layout::from_schema(world.schema(id).map_err(err)?, kind);
        layouts.insert(layout.schema.clone(), layout);
    }

    // The modules' systems before the game's (physics' collider recording,
    // the UI's begin), then the game's, then the modules' after them.
    let ui = super::modules::new_ui(game.tick, previous.as_ref().map(|p| p.0));
    let physics = super::modules::mount_front(&mut world, engine_id, profile, &ui)?;

    // Systems: the game's, ordered.
    let mut systems_decl = Vec::new();
    for &s in &game.systems {
        systems_decl.push(
            all_systems
                .get(s as usize)
                .cloned()
                .ok_or_else(|| format!("the game lists system {s}, which was never declared"))?,
        );
    }
    let mut systems = Vec::new();
    for i in order(&systems_decl)? {
        let s = &systems_decl[i];
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
                let layout = layouts.get(name).ok_or_else(|| {
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
        let index = systems.len();
        world
            .add_system(desc, move |ctx| run_system(engine_id, index, ctx))
            .map_err(err)?;
        systems.push(SystemEntry {
            name: s.name.clone(),
            callback: s.run,
            reads: names(&s.reads),
            writes: names(&s.writes),
            columns,
            matched: 0,
            ms: 0.0,
        });
    }
    let mut all_commands = commands;
    for (name, handler) in &game.commands {
        all_commands.insert(
            name.clone(),
            CommandDecl {
                handler: *handler,
                usage: String::new(),
                doc: String::new(),
            },
        );
    }
    // og-anim after the game's systems (they set players, parameters and IK
    // targets), before the UI's end and physics (root motion moves Transforms).
    // Tweens and flipbooks, then og-anim, then tilemaps and camera feel;
    // og.sprite after physics (the poses at the tick's end).
    let components: Vec<String> = layouts
        .values()
        .filter(|l| l.kind == RecordKind::Component)
        .map(|l| l.schema.clone())
        .collect();
    let feel = super::feel::install_front(&mut world, &components)?;
    let anim = super::anim::install(&mut world)?;
    let feel_records = super::feel::Records::of(&world, &layouts)?;
    super::feel::install_middle(&mut world, &feel, &feel_records)?;
    let mounted =
        super::modules::mount_back(&mut world, engine_id, physics, ui, previous.map(|p| p.1))?;
    super::feel::install_back(&mut world, &feel, &feel_records)?;
    let names_by_id = world.schemas().into_iter().map(|s| s.name).collect();
    Ok(Assembled {
        world,
        probe: Some(probe),
        layouts,
        names_by_id,
        resources,
        systems,
        commands: all_commands,
        warnings,
        game,
        mounted,
        anim,
        feel,
        feel_records,
    })
}

/// The layouts extraction reads, from an engine's registered records.
pub(crate) fn extract_layouts(eng: &Engine) -> (Option<Layout>, Option<Layout>, DrawLayouts) {
    let st = eng.state.borrow();
    let get = |n: &str| st.layouts.get(n).cloned();
    (
        get("og.Camera"),
        get("og.RenderSettings"),
        DrawLayouts {
            model: get("og.Model"),
            mesh: get("og.Mesh"),
            directional: get("og.DirectionalLight"),
            point: get("og.PointLight"),
            spot: get("og.SpotLight"),
            fx: super::fx::FxLayouts::from(get),
            atmo: super::atmosphere::AtmoLayouts::from(get, st.physics.clone()),
        },
    )
}

fn create_inner(game: u32, options: &str) -> Result<u32, String> {
    let options = parse(Some(options))?;
    let engine_id = ENGINES.with(|e| e.borrow().len() as u32);
    let seed = options["seed"].as_u64().map(|s| s as u32);
    let profile = match options["profile"].as_str() {
        None | Some("3d") => og_physics::PROFILE_3D,
        Some("2d") => og_physics::PROFILE_2D,
        Some(other) => {
            return Err(format!(
                "options.profile is '{other}'; a project's profile is \"2d\" or \"3d\""
            ));
        }
    };
    let mut a = assemble(game, seed, engine_id, profile, None)?;
    let probe = a.probe.take().expect("assemble makes a probe");
    let state = State {
        layouts: a.layouts,
        names_by_id: a.names_by_id,
        resources: a.resources,
        systems: a.systems,
        commands: a.commands,
        names: BTreeMap::new(),
        name_of: BTreeMap::new(),
        name_ops: Vec::new(),
        rate: a.game.tick,
        max_catch_up: a.game.max_catch_up.max(1),
        accumulator_ms: 0.0,
        facts: a.game.facts,
        boot_scene: None,
        start: a.game.start,
        step_ms: 0.0,
        render_ms: 0.0,
        warnings: a.warnings,
        log: Vec::new(),
        outputs: Vec::new(),
        reloaded: None,
        profile,
        physics: Some(a.mounted.physics.clone()),
        ui: Some(a.mounted.ui.clone()),
        physics_meshes: BTreeMap::new(),
        ui_warned: BTreeSet::new(),
        anim: Some(a.anim.clone()),
        anim_assets: Default::default(),
        feel: Some(a.feel.clone()),
        feel_assets: Default::default(),
    };
    og_audio::publish_link(engine_id, a.mounted.audio.clone());
    let render = Rc::new(RefCell::new(RenderState::new(a.game.render.clone())));
    render.borrow_mut().ui = Some(a.mounted.ui.clone());
    let engine = Rc::new(Engine {
        id: engine_id,
        world: UnsafeCell::new(a.world),
        stepping: Cell::new(false),
        state: RefCell::new(state),
        probe: RefCell::new(probe),
        render,
        assets: RefCell::new(Assets {
            root: options["root"].as_str().unwrap_or_default().to_string(),
            ..Assets::default()
        }),
    });
    {
        let (camera, settings, draw) = extract_layouts(&engine);
        let mut rs = engine.render.borrow_mut();
        rs.camera = camera;
        rs.settings = settings;
        rs.draw = draw;
        rs.fx.anim = Some(a.anim.clone());
        rs.feel.feel = Some((a.feel.clone(), a.feel_records.clone()));
    }
    rprobe::install(&engine, a.game.facts)?;
    ENGINES.with(|e| e.borrow_mut().push(Some(engine.clone())));
    rprobe::set_reloader(&engine);

    // The game's bundle and its first scene, before the first tick.
    if !engine.assets.borrow().root.is_empty() {
        let i = engine.assets.borrow_mut().open_bundle("cooked");
        engine.assets.borrow_mut().default_bundle = Some(i);
    }
    if let Some(path) = a.game.scenes.get(&a.game.scene) {
        engine.state.borrow_mut().boot_scene = Some(path.clone());
        boot_scene(&engine);
    }
    // `deferStart`: the host attaches its renderer and reports the viewport
    // first; start then runs at the first tick boundary.
    if !options["deferStart"].as_bool().unwrap_or(false) {
        run_start(&engine)?;
    }
    Ok(engine_id)
}

/// GameDesc.start, once, when the first scene is in (or there is none).
fn run_start(eng: &Engine) -> Result<(), String> {
    if eng.state.borrow().boot_scene.is_some() {
        return Ok(());
    }
    let Some(start) = eng.state.borrow_mut().start.take() else {
        return Ok(());
    };
    before_structural(eng);
    call(eng, 1, start, "[]".into()).map_err(|e| format!("GameDesc.start: {e}"))?;
    Ok(())
}

/// Spawns GameDesc.scene once its bundle is ready; a failed bundle is a
/// warning and the world starts without it.
fn boot_scene(eng: &Engine) {
    let Some(path) = eng.state.borrow().boot_scene.clone() else {
        return;
    };
    let status = {
        let assets = eng.assets.borrow();
        match assets.default_bundle {
            None => Status::Failed(format!(
                "scene '{path}' was not loaded: the host gave no project root, so no cooked bundle is open"
            )),
            Some(i) => assets.entries[i].status.clone(),
        }
    };
    let result = match status {
        Status::Pending => return,
        Status::Failed(e) => Err(e),
        Status::Ready => super::assets::load_scene(eng, &path).and_then(|i| {
            let scene = match &eng.assets.borrow().entries[i].payload {
                super::assets::Payload::Scene(s) => s.clone(),
                _ => return Err(format!("{path} is not a scene")),
            };
            super::assets::spawn_scene(eng, &scene, None).map(|_| ())
        }),
    };
    let mut st = eng.state.borrow_mut();
    st.boot_scene = None;
    if let Err(e) = result {
        st.warnings
            .push(format!("GameDesc.scene: {e}; the world starts without it"));
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": format!("GameDesc.scene: {e}") }));
    }
    let warnings = std::mem::take(&mut eng.assets.borrow_mut().warnings);
    for w in warnings {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": w }));
    }
}

/// `og_engine_destroy`.
pub fn destroy(id: u32) -> bool {
    let Ok(eng) = engine(id) else { return false };
    if eng.stepping.get() {
        return false;
    }
    before_structural(&eng);
    og_audio::retire_link(id);
    ENGINES.with(|e| {
        if let Some(slot) = e.borrow_mut().get_mut(id as usize) {
            *slot = None;
        }
    });
    true
}

/// Attach (or detach, with `None`) a renderer to an engine; the probe's
/// `capture` draws through it.
pub fn set_render_backend(id: u32, backend: Option<Box<dyn RenderBackend>>) -> Result<(), String> {
    let eng = engine(id)?;
    rprobe::attach(&eng, backend);
    Ok(())
}

/// The game's RenderOptions (`GameDesc.render`), for a host creating a renderer.
pub fn render_options(id: u32) -> Value {
    engine(id).map_or(Value::Null, |e| e.render.borrow().options.clone())
}

// ── the tick ────────────────────────────────────────────────────────────────

/// At a tick boundary: pending bundles, a scene waiting for its bundle.
fn tick_boundary(eng: &Engine) {
    eng.assets.borrow_mut().poll();
    if eng.state.borrow().boot_scene.is_some() {
        boot_scene(eng);
    }
    if let Err(e) = run_start(eng) {
        let mut st = eng.state.borrow_mut();
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": e }));
    }
}

/// Up to `steps` ticks through the probe (due command steps run on the
/// way). `Err` names the tick and the failing system.
fn advance_probe(eng: &Engine, steps: u64) -> Result<u64, String> {
    tick_boundary(eng);
    let t0 = rprobe::now_ms();
    let mut left = steps;
    let tick = loop {
        let chunk = left.min(og_probe::PLAYTEST_MAX_TICKS);
        let stepped = rprobe::with_probe(eng, false, |p, w| p.advance(w, chunk));
        super::modules::end_of_ticks();
        let tick = stepped?.map_err(err)?;
        left -= chunk;
        if left == 0 || eng.probe.borrow().quit_requested() {
            break tick;
        }
    };
    if steps > 0 {
        eng.state.borrow_mut().step_ms = (rprobe::now_ms() - t0) / steps as f64;
    }
    Ok(tick)
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
    match advance_probe(&eng, u64::from(steps)) {
        Ok(tick) => ok(json!(tick)),
        Err(e) => refuse(e),
    }
}

/// `og_engine_frame`.
pub fn frame(id: u32, elapsed_ms: f64) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => return refuse(e),
    };
    if eng.stepping.get() {
        return refuse("frame was called from inside a system");
    }
    let paused = eng.probe.borrow().paused();
    let due = {
        let mut st = eng.state.borrow_mut();
        if paused || !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
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
    tick_boundary(&eng);
    let t0 = rprobe::now_ms();
    let stepped = rprobe::with_probe(&eng, false, |p, w| p.frame(w, due));
    super::modules::end_of_ticks();
    let ticks = match stepped {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return refuse(err(e)),
        Err(e) => return refuse(e),
    };
    if ticks > 0 {
        eng.state.borrow_mut().step_ms = (rprobe::now_ms() - t0) / f64::from(ticks);
    }
    let rendered = match render(&eng) {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    let probe = eng.probe.borrow();
    json!({
        "ok": true, "ticks": ticks, "tick": tick, "rendered": rendered,
        "paused": probe.paused(), "quit": probe.quit_requested(),
    })
    .to_string()
}

fn render(eng: &Engine) -> Result<bool, String> {
    if eng.render.borrow().backend.is_none() {
        return Ok(false);
    }
    let t0 = rprobe::now_ms();
    let notes = {
        let world = eng.world()?;
        let mut rs = eng.render.borrow_mut();
        super::extract::extract(world, &mut rs);
        let rs = &mut *rs;
        if let Some(b) = rs.backend.as_mut() {
            b.render(&rs.scene)?;
        }
        std::mem::take(&mut rs.notes)
    };
    let mut st = eng.state.borrow_mut();
    st.render_ms = rprobe::now_ms() - t0;
    for n in notes {
        if st.ui_warned.insert(n.clone()) {
            st.log
                .push(json!({ "tick": 0, "warning": true, "text": n }));
        }
    }
    Ok(true)
}

/// `og_engine_call`: the probe's JSON-RPC entry.
pub fn call_rpc(id: u32, frame: &str) -> String {
    match og_probe::rpc::parse(frame) {
        Ok(call) => answer_call(id, &call),
        Err(error) => error,
    }
}

/// `og_engine_take_outputs`.
pub fn take_outputs(id: u32) -> String {
    match engine(id) {
        Ok(eng) => {
            let outputs = std::mem::take(&mut eng.state.borrow_mut().outputs);
            rprobe::outputs_json(outputs).to_string()
        }
        Err(_) => "[]".into(),
    }
}

/// `og_engine_take_log`.
pub fn take_log(id: u32) -> String {
    match engine(id) {
        Ok(eng) => {
            let mut st = eng.state.borrow_mut();
            let mut log = std::mem::take(&mut st.log);
            for w in std::mem::take(&mut eng.assets.borrow_mut().warnings) {
                log.push(json!({ "tick": 0, "warning": true, "text": w }));
            }
            Value::Array(log).to_string()
        }
        Err(_) => "[]".into(),
    }
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
    let was_stepping = eng.stepping.replace(true);
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
            payload: super::feel::time_payload(ctx),
        }),
        None => Err("no dispatcher is installed (og_host_set_dispatcher)".into()),
    };
    FRAME.with(|f| f.set(prev));
    eng.stepping.set(was_stepping);
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
    if let Some(f) = running(world) {
        // Inside a system: through its context, which refuses a resource
        // the system did not declare among its writes.
        let result = (|| {
            let eng = engine(world)?;
            let value = parse(Some(value))?;
            let (layout, id) = {
                let st = eng.state.borrow();
                let layout = st
                    .layouts
                    .get(resource)
                    .filter(|l| l.kind == RecordKind::Resource)
                    .ok_or_else(|| format!("'{resource}' is not a registered resource"))?
                    .clone();
                let id = st
                    .names_by_id
                    .iter()
                    .position(|n| n == resource)
                    .map(|i| SchemaId::from_index(i as u32))
                    .ok_or_else(|| format!("'{resource}' is not registered with this world"))?;
                (layout, id)
            };
            if let Some(field) = layout
                .fields
                .iter()
                .find(|fl| fl.readonly && value.get(&fl.name).is_some())
            {
                return Err(format!("{resource}.{} is read only", field.name));
            }
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let current = ctx.resource_bytes(id).map_err(err)?;
            let bytes = layout.encode(Some(&current), Some(&value))?;
            ctx.set_resource_bytes(id, &bytes).map_err(err)
        })();
        return match result {
            Ok(()) => super::ok_void(),
            Err(e) => refuse(e),
        };
    }
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

/// Answers one parsed JSON-RPC call (a host's own transport: the native
/// WebSocket server's `Pending::call`), exactly as [`call_rpc`] does.
pub fn answer_call(id: u32, call: &og_probe::rpc::Call) -> String {
    let eng = match engine(id) {
        Ok(e) => e,
        Err(e) => {
            return json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string()
        }
    };
    tick_boundary(&eng);
    let answer = rprobe::with_probe(&eng, true, |p, w| rprobe::answer(p, w, call));
    let swapped = answer.as_ref().is_ok_and(|a| a.1);
    rprobe::after_reload(&eng, swapped);
    rprobe::resync_names(&eng);
    match answer {
        Ok(a) => a.0,
        Err(e) => json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string(),
    }
}
