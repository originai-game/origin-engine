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
    /// A streaming group's pack (`cooked/stream/<name>.ogb`) is read once
    /// per load: a host may hand it over and forget it.
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: this host reads no files"))
    }
    /// A streaming group let go of its pack `pack` (it was unloaded, or the
    /// pack failed): a host that keeps fetched packs and transcoded
    /// textures drops the pack and the group's `textures` (asset ids) now,
    /// and fetches them again when the group loads again.
    fn release_stream(&self, pack: &str, textures: &[String]) {
        let _ = (pack, textures);
    }
    /// A cooked texture for the renderer, in the device's encoding when
    /// `formats` (the attached renderer's) names one: og-cook's transcoder
    /// (native, feature `transcode`) by default, every mip level; the web
    /// host takes what its Worker transcoded with Binomial's Basis
    /// transcoder. RGBA8 without a renderer.
    fn texture(
        &self,
        texture: &og_cook::runtime::CookedTexture,
        formats: Option<&og_render::TextureFormats>,
    ) -> Fetch<og_render::TextureSource> {
        match texture.to_texture_source(formats) {
            Ok(t) => Fetch::Ready(t),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
    /// Starts inflating zlib bytes away from the engine's thread (the web
    /// host's DecompressionStream), answered later by
    /// [`Hooks::inflate_take`] under `key`. `false`: this host cannot, and
    /// the runtime inflates on its own (a worker thread natively).
    fn inflate_start(&self, key: u64, zlib: &[u8]) -> bool {
        let _ = (key, zlib);
        false
    }
    /// The bytes a started inflate answered, once (`Pending` until then).
    fn inflate_take(&self, key: u64) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("inflate {key} was never started"))
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
    /// GameDesc.commandHelp: each action's usage and doc.
    pub command_help: BTreeMap<String, (String, String)>,
    pub max_catch_up: u32,
    pub facts: Option<u32>,
    pub scene: String,
    pub scenes: BTreeMap<String, String>,
    /// GameDesc.render (RenderOptions), as given.
    pub render: Value,
    /// GameDesc.start, called once before the first tick.
    pub start: Option<u32>,
    /// GameDesc.look: applied before start runs; empty is none.
    pub look: String,
    /// Which of tick, seed, scenes and scene GameDesc set: the others take
    /// project.ogt's ([`with_project`]), else their defaults.
    pub given: Given,
}

/// Which of GameDesc's tick, seed, scenes and scene the game set.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Given {
    pub tick: bool,
    pub seed: bool,
    pub scenes: bool,
    pub scene: bool,
}

/// project.ogt's `tick`, `seed`, `scenes` and `scene`, as the host read
/// them (CreateOptions.project): the defaults of the GameDesc fields a game
/// leaves unset.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ProjectGame {
    pub tick: Option<u32>,
    pub seed: Option<u32>,
    pub scenes: Option<BTreeMap<String, String>>,
    pub scene: Option<String>,
}

/// Reads `tick`, `seed`, `scenes` and `scene` from project.ogt's text; a
/// value of the wrong kind is refused naming its line. Other keys are og's
/// (`name`, `entry`, `profile`, `publish`, ...) and are not read here.
pub(crate) fn project_game(text: &str) -> Result<ProjectGame, String> {
    let doc = og_core::ogt::parse(text).map_err(|e| format!("project.ogt: {}", e.reason()))?;
    let b = &doc.body;
    let whole = |key: &str, lo: u64, hi: u64, what: &str| -> Result<Option<u32>, String> {
        let Some(e) = b.get(key) else {
            return Ok(None);
        };
        e.number(0)
            .ok()
            .and_then(|n| n.as_u64().ok())
            .filter(|n| (lo..=hi).contains(n))
            .map(|n| Some(n as u32))
            .ok_or_else(|| format!("project.ogt:{}: `{key}` takes {what}", e.line))
    };
    let tick = whole(
        "tick",
        1,
        240,
        "a whole number of ticks per second, 1 to 240",
    )?;
    let seed = whole(
        "seed",
        0,
        u64::from(u32::MAX),
        "a whole number from 0 to 4294967295",
    )?;
    let scene = match b.get("scene") {
        None => None,
        Some(e) => Some(
            e.values
                .first()
                .and_then(|v| v.as_word().or_else(|| v.as_str()))
                .ok_or_else(|| {
                    format!(
                        "project.ogt:{}: `scene` names a scene of `scenes`, such as `scene main`",
                        e.line
                    )
                })?
                .to_string(),
        ),
    };
    let scenes = match b.get("scenes") {
        None => None,
        Some(e) => {
            let block = e.block.as_ref().ok_or_else(|| {
                format!(
                    "project.ogt:{}: `scenes` is a block of `<name> \"<path>\"` lines",
                    e.line
                )
            })?;
            let mut out = BTreeMap::new();
            for s in &block.entries {
                let path = s.string(0).map_err(|_| {
                    format!("project.ogt:{}: scenes.{} takes the scene's quoted path, such as \"scenes/main.ogt\"", s.line, s.key)
                })?;
                out.insert(s.key.clone(), path.to_string());
            }
            Some(out)
        }
    };
    Ok(ProjectGame {
        tick,
        seed,
        scenes,
        scene,
    })
}

/// GameDesc with project.ogt's values for the fields it left unset. A field
/// set in both places to different values is refused: one value has one
/// home, and the two would drift apart silently.
pub(crate) fn with_project(mut game: GameDecl, p: &ProjectGame) -> Result<GameDecl, String> {
    fn both<T: PartialEq + Clone + std::fmt::Debug>(
        field: &str,
        given: bool,
        ours: &mut T,
        theirs: &Option<T>,
    ) -> Result<(), String> {
        match theirs {
            None => Ok(()),
            Some(v) if !given => {
                *ours = v.clone();
                Ok(())
            }
            Some(v) if v == ours => Ok(()),
            Some(v) => Err(format!(
                "GameDesc.{field} is {ours:?} but project.ogt's `{field}` is {v:?}: one value set in two places; keep one (delete the line from project.ogt, or the field from GameDesc)"
            )),
        }
    }
    let g = game.given;
    both("tick", g.tick, &mut game.tick, &p.tick)?;
    both("seed", g.seed, &mut game.seed, &p.seed)?;
    both("scenes", g.scenes, &mut game.scenes, &p.scenes)?;
    both("scene", g.scene, &mut game.scene, &p.scene)?;
    Ok(game)
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
    /// The columns borrowed so far: a running system's are borrowed when
    /// the view's `column` first asks for them (`lazy`), a world query's
    /// when the view is made.
    pub columns: Vec<(String, ColumnPtr)>,
    pub types: Vec<String>,
    /// A running system's view: its archetype's place among the system's
    /// matched archetypes, and the columns it may borrow from it.
    pub lazy: Option<(usize, Vec<QueryColumn>)>,
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
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut T> {
        handle
            .checked_sub(self.base)
            .and_then(|i| self.items.get_mut(i as usize))
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
    /// The asset id of the scene GameDesc.scene spawned (`save_scene`
    /// writes its source back).
    pub scene_source: Option<String>,
    /// GameDesc.start, until it has run.
    pub start: Option<u32>,
    /// What GameDesc.start threw: the world is half-built, so every later
    /// advance, frame and probe call is refused with it.
    pub start_failed: Option<String>,
    /// project.ogt's game values (CreateOptions.project), for a reload's
    /// assembly.
    pub project: Option<ProjectGame>,
    /// GameDesc.look, until it has been applied (just before start).
    pub boot_look: Option<String>,
    /// The last UiDesc given to og.ui.configure, which applying a look's
    /// skin keeps.
    pub ui_desc: Value,
    pub step_ms: f64,
    pub render_ms: f64,
    /// The last frame's render-graph passes and their encode milliseconds.
    pub pass_ms: Vec<(String, f64)>,
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
    /// Every terrain's height field registered with physics, likewise.
    pub physics_heightfields: BTreeMap<u64, og_physics::HeightfieldDesc>,
    /// UI warnings already logged, so a repeated one is logged once.
    pub ui_warned: BTreeSet<String>,
    /// The host mirrors the UI's accessibility tree (create option `a11y`,
    /// the web Worker): frame answers carry the tree when it changed.
    pub a11y_mirror: bool,
    /// The digest of the accessibility tree the last frame answer carried
    /// (`A11yTree::digest`; 0: none sent yet), so the mirror is sent the
    /// tree only when it changes.
    pub a11y_digest: u64,
    /// Undeclared accesses already logged (`system access name`), so each is
    /// logged once per system (`api::undeclared`).
    pub undeclared_logged: BTreeSet<String>,
    /// og-anim on this world, and its assets (replayed into a reload's world).
    pub anim: Option<og_anim::AnimationSystem>,
    pub anim_assets: super::anim::AnimAssets,
    /// Tweens, flipbooks, tilemaps, camera feel and sprites on this world,
    /// and their assets (replayed into a reload's world).
    pub feel: Option<super::feel::Feel>,
    pub feel_assets: super::feel::FeelAssets,
    /// A world.restore asked for inside a tick, applied when it ends.
    pub pending_restore: Option<super::save::Pending>,
    /// A restore applied since the last tick began, (the tick the world
    /// was at, the saved tick): og.WorldRestored is sent as the next tick
    /// starts, so the world between ticks hashes as the save did.
    pub restored: Option<(u64, u64)>,
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
    /// The column borrows its views took (`view_columns`), held until the
    /// system returns: a column is borrowed only once game code takes a
    /// view of it, so `world.get`/`world.set` reach every column no view
    /// holds (kestrel-stair FINDINGS #6).
    pub held: RefCell<Vec<Held>>,
}

/// One column borrow a view holds. The lifetime is the world's for the
/// system call: [`run_system`] drops every one before it returns.
pub(crate) enum Held {
    Read(#[allow(dead_code)] std::cell::Ref<'static, [u8]>),
    Write(#[allow(dead_code)] std::cell::RefMut<'static, [u8]>),
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
        command_help: desc["commandHelp"]
            .as_object()
            .map(|o| {
                o.iter()
                    .map(|(k, v)| {
                        let text = |f: &str| v[f].as_str().unwrap_or_default().to_string();
                        (k.clone(), (text("usage"), text("doc")))
                    })
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
        look: desc["look"].as_str().unwrap_or_default().to_string(),
        given: Given {
            tick: desc["tick"].is_u64(),
            seed: desc["seed"].is_u64(),
            scenes: desc["scenes"].is_object(),
            scene: desc["scene"].is_string(),
        },
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
    project: Option<&ProjectGame>,
    previous: Option<(&og_ui::SharedUi, og_audio::AudioLink)>,
) -> Result<Assembled, String> {
    bump_layout_epoch();
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
    let game = match project {
        Some(p) => with_project(game, p)?,
        None => game,
    };
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
    // Systems timed by name for the probe's dump_world and og run --json
    // (measurement only: nothing in the tick reads it).
    world.set_clock(Some(rprobe::now_ms));
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
            || super::timeline::REGISTERED_BY_TIMELINE.contains(&decl.schema)
            || decl.schema == super::stream::EVENT
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
        world.register::<og_anim::Skeleton2DEvent>().map_err(err)?;
        for name in super::feel::REGISTERED_BY_FEEL {
            let id = world.id_by_name(name).map_err(err)?;
            let layout = Layout::from_schema(world.schema(id).map_err(err)?, RecordKind::Event);
            layouts.insert(layout.schema.clone(), layout);
        }
    }
    // og.timeline's events (the marker's name is a str64).
    super::timeline::register_records(&mut world, &mut layouts)?;
    // og.stream's event (its group is a str64), registered as it is sent.
    {
        world
            .register::<super::stream::StreamEvent>()
            .map_err(err)?;
        let id = world.id_by_name(super::stream::EVENT).map_err(err)?;
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
    let record_names: Vec<String> = layouts.keys().cloned().collect();
    super::terrain::mount(&mut world, engine_id)?;
    super::e6::mount_front(&mut world, engine_id, profile, &physics, &record_names)?;

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
                for f in layout.fields.iter() {
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
        let (usage, doc) = game.command_help.get(name).cloned().unwrap_or_default();
        all_commands.insert(
            name.clone(),
            CommandDecl {
                handler: *handler,
                usage,
                doc,
            },
        );
    }
    for name in game.command_help.keys() {
        if !game.commands.iter().any(|(n, _)| n == name) {
            warnings.push(format!(
                "GameDesc.commandHelp names '{name}', which GameDesc.commands does not declare"
            ));
        }
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
    super::video::install(&mut world, engine_id, &layouts)?;
    // og.path and og.timeline after tweens, before og-anim (an animation
    // track sets players) and physics.
    super::timeline::install(&mut world, engine_id, &layouts, &ui)?;
    let anim = super::anim::install(&mut world)?;
    let feel_records = super::feel::Records::of(&world, &layouts)?;
    super::feel::install_middle(&mut world, engine_id, &feel, &feel_records)?;
    super::e6::mount_back(&mut world, engine_id, profile, &physics)?;
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
            environment: get("og.Environment"),
            terrain: get("og.Terrain"),
            scatter: get("og.TerrainScatter"),
            collider: get("og.Collider"),
            lines: get("og.Lines"),
            points: get("og.Points"),
            physics: st.physics.clone(),
            engine: eng.id,
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
    // project.ogt's tick, seed, scenes and scene: what GameDesc leaves unset.
    let project = match options["project"].as_str() {
        Some(text) => Some(project_game(text)?),
        None => None,
    };
    let mut a = assemble(game, seed, engine_id, profile, project.as_ref(), None)?;
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
        scene_source: None,
        start: a.game.start,
        start_failed: None,
        project,
        boot_look: (!a.game.look.is_empty()).then(|| a.game.look.clone()),
        ui_desc: Value::Null,
        step_ms: 0.0,
        render_ms: 0.0,
        pass_ms: Vec::new(),
        // A GameDesc mistake is logged at once (the others answer the call that meets them).
        log: a
            .warnings
            .iter()
            .filter(|w| w.starts_with("GameDesc."))
            .map(|w| json!({ "tick": 0, "warning": true, "text": w }))
            .collect(),
        warnings: a.warnings,
        outputs: Vec::new(),
        reloaded: None,
        profile,
        physics: Some(a.mounted.physics.clone()),
        ui: Some(a.mounted.ui.clone()),
        physics_meshes: BTreeMap::new(),
        physics_heightfields: BTreeMap::new(),
        ui_warned: BTreeSet::new(),
        a11y_mirror: options["a11y"].as_bool().unwrap_or(false),
        a11y_digest: 0,
        undeclared_logged: BTreeSet::new(),
        anim: Some(a.anim.clone()),
        anim_assets: Default::default(),
        feel: Some(a.feel.clone()),
        feel_assets: Default::default(),
        pending_restore: None,
        restored: None,
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
    {
        let mut probe = engine.probe.borrow_mut();
        super::e6::commit(engine_id, probe.inbox());
        super::e6::register_commands(&mut probe, engine_id);
        super::timeline::commit(engine_id, probe.inbox());
        super::timeline::register_commands(&mut probe, engine_id);
    }
    // The host's locale (og run --locale, the browser's language).
    super::i18n::request(engine_id, options["locale"].as_str().unwrap_or(""));
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
    // GameDesc.look's antialias, before the host attaches its renderer.
    if !a.game.look.is_empty() {
        super::look::boot_antialias(&engine, &a.game.look);
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
    let look = eng.state.borrow_mut().boot_look.take();
    if let Some(path) = look
        && let Err(e) = super::look::load(eng, &path).and_then(|i| super::look::apply(eng, i))
    {
        let text = format!("GameDesc.look: {e}; the world starts without it");
        let mut st = eng.state.borrow_mut();
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
        st.warnings.push(text);
    }
    let Some(start) = eng.state.borrow_mut().start.take() else {
        return Ok(());
    };
    before_structural(eng);
    if let Err(e) = call(eng, 1, start, "[]".into()) {
        let text =
            format!("GameDesc.start threw, so the world is half-built and does not run: {e}");
        eng.state.borrow_mut().start_failed = Some(text.clone());
        return Err(text);
    }
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
            super::assets::spawn_scene(eng, &scene, None).map(|_| {
                eng.state.borrow_mut().scene_source = Some(scene.id.clone());
            })
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
    super::e6::retire(id);
    super::timeline::retire(id);
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

/// Make one pipeline of the engine's renderer that no frame has used yet
/// (og-render's `Renderer::warm`), when the host has idle time: the web
/// page behind its play gate. False when every one is made, or without a
/// renderer.
pub fn warm_renderer(id: u32) -> bool {
    engine(id).is_ok_and(|e| {
        e.render
            .try_borrow_mut()
            .is_ok_and(|mut rs| rs.backend.as_mut().is_some_and(|b| b.warm()))
    })
}

/// The game's RenderOptions (`GameDesc.render`), for a host creating a renderer.
pub fn render_options(id: u32) -> Value {
    engine(id).map_or(Value::Null, |e| e.render.borrow().options.clone())
}

/// `base` with the game's RenderOptions applied (GameDesc.render):
/// antialias, shadowMapSize, pointShadows, taaHistory and taaSharpen. Both
/// hosts create their renderer from it, so the options mean the same on
/// each.
pub fn render_desc(id: u32, base: og_render::RenderDesc) -> og_render::RenderDesc {
    let options = render_options(id);
    let mut desc = base;
    desc.antialias = match options["antialias"].as_str().unwrap_or("tier") {
        "off" => og_render::Antialias::Off,
        "msaa4" => og_render::Antialias::Msaa4,
        "taa" => og_render::Antialias::Taa,
        "fxaa" => og_render::Antialias::Fxaa,
        _ => og_render::Antialias::Tier,
    };
    if let Some(size) = options["shadowMapSize"].as_u64() {
        desc.shadow_map_size = size.clamp(256, 4096) as u32;
    }
    desc.point_shadows = match options["pointShadows"].as_str().unwrap_or("auto") {
        "cube" => og_render::PointShadows::Cube,
        "paraboloid" => og_render::PointShadows::Paraboloid,
        _ => og_render::PointShadows::Auto,
    };
    desc.taa_history = match options["taaHistory"].as_str() {
        Some("catmullRom") => og_render::TaaHistory::CatmullRom,
        _ => og_render::TaaHistory::Bilinear,
    };
    desc.taa_sharpen = options["taaSharpen"]
        .as_f64()
        .map_or(0.0, |v| v.clamp(0.0, 1.0) as f32);
    desc
}

/// og-render's Antialias by its IDL name (RenderCapabilities.antialias).
pub fn antialias_name(a: og_render::Antialias) -> &'static str {
    match a {
        og_render::Antialias::Msaa4 => "msaa4",
        og_render::Antialias::Taa => "taa",
        og_render::Antialias::Fxaa => "fxaa",
        og_render::Antialias::Off | og_render::Antialias::Tier => "off",
    }
}

/// og-render's PointShadows by its IDL name (RenderCapabilities.pointShadows).
pub fn point_shadows_name(p: og_render::PointShadows) -> &'static str {
    match p {
        og_render::PointShadows::Cube => "cube",
        og_render::PointShadows::Paraboloid => "paraboloid",
        og_render::PointShadows::Auto => "auto",
    }
}

// ── the tick ────────────────────────────────────────────────────────────────

/// At a tick boundary: pending bundles, a scene waiting for its bundle,
/// GameDesc.start. `Err` once GameDesc.start has thrown, at this boundary
/// or an earlier one: no tick runs on the half-built world.
fn tick_boundary(eng: &Engine) -> Result<(), String> {
    eng.assets.borrow_mut().poll();
    super::stream::update(eng);
    super::stream::sound_chunks(eng);
    super::stream::resolve(eng);
    if eng.state.borrow().boot_scene.is_some() {
        boot_scene(eng);
    }
    if let Some(e) = &eng.state.borrow().start_failed {
        return Err(e.clone());
    }
    run_start(eng)
}

/// The wall time since `t0` the ticks took, less the probe's screenshots
/// taken on the way (a 1280x720 PNG is a fifth of a second in a dev build:
/// a step time that counted it read as a stall on every screenshot tick).
fn step_ms(eng: &Engine, t0: f64) -> f64 {
    (rprobe::now_ms() - t0 - eng.render.borrow().capture_ms).max(0.0)
}

/// Up to `steps` ticks through the probe (due command steps run on the
/// way). `Err` names the tick and the failing system.
fn advance_probe(eng: &Engine, steps: u64) -> Result<u64, String> {
    tick_boundary(eng)?;
    eng.render.borrow_mut().capture_ms = 0.0;
    let t0 = rprobe::now_ms();
    let mut left = steps;
    let tick = loop {
        let chunk = left.min(og_probe::PLAYTEST_MAX_TICKS);
        let stepped = rprobe::with_probe(eng, false, |p, w| p.advance(w, chunk));
        super::modules::end_of_ticks();
        super::stream::sound_chunks(eng);
        let tick = stepped?.map_err(err)?;
        left -= chunk;
        if left == 0 || eng.probe.borrow().quit_requested() {
            break tick;
        }
    };
    if steps > 0 {
        eng.state.borrow_mut().step_ms = step_ms(eng, t0) / steps as f64;
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
    if let Err(e) = tick_boundary(&eng) {
        return refuse(e);
    }
    eng.render.borrow_mut().capture_ms = 0.0;
    let t0 = rprobe::now_ms();
    let stepped = rprobe::with_probe(&eng, false, |p, w| p.frame(w, due));
    super::modules::end_of_ticks();
    super::stream::sound_chunks(&eng);
    let ticks = match stepped {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => return refuse(err(e)),
        Err(e) => return refuse(e),
    };
    if ticks > 0 {
        eng.state.borrow_mut().step_ms = step_ms(&eng, t0) / f64::from(ticks);
    }
    let rendered = match render(&eng) {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    let (behind, step_ms, render_ms) = {
        let st = eng.state.borrow();
        (st.accumulator_ms, st.step_ms, st.render_ms)
    };
    let (text, copy) = text_entry(&eng);
    let a11y = a11y_changed(&eng);
    let probe = eng.probe.borrow();
    // `behind`: real time the accumulator holds that no tick has simulated
    // yet, so a host can place an input event within the next tick.
    // `text`: the focused text control (where the page's text target goes,
    // the selection a copy takes), `copy`: text a Ctrl/Cmd+C or X took.
    // `stepMs`: wall milliseconds a tick took (the mean of this frame's, or
    // the last frame's that ticked), `renderMs`: the last render's.
    // `pending`: the command channel's steps still to run (a script has
    // ended when it is 0, wherever a restore moved og.Time).
    let mut answer = json!({
        "ok": true, "ticks": ticks, "tick": tick, "rendered": rendered,
        "paused": probe.paused(), "quit": probe.quit_requested(), "behind": behind,
        "text": text, "stepMs": step_ms, "renderMs": render_ms,
        "pending": probe.steps_left(),
    });
    if let Some(c) = copy {
        answer["copy"] = json!(c);
    }
    // `a11y`: the UI's accessibility tree (as og.ui.a11y answers it), on
    // the frames where it changed, for a host that mirrors it (create
    // option `a11y`: the web shell's DOM mirror).
    if let Some(tree) = a11y {
        answer["a11y"] = tree;
    }
    answer.to_string()
}

/// The UI's accessibility tree when it differs from the one the last frame
/// answer carried (by [`og_ui::A11yTree::digest`]).
fn a11y_changed(eng: &Engine) -> Option<Value> {
    if !eng.state.borrow().a11y_mirror {
        return None;
    }
    let ui = eng.state.borrow().ui.clone()?;
    let ui = ui.lock();
    let tree = ui.a11y();
    let digest = tree.digest().max(1);
    let mut st = eng.state.borrow_mut();
    if st.a11y_digest == digest {
        return None;
    }
    st.a11y_digest = digest;
    Some(super::modules::a11y_json(tree))
}

/// The UI's focused text control as the web shell takes it (`null`: none),
/// and the text a copy or cut took since the last frame.
fn text_entry(eng: &Engine) -> (Value, Option<String>) {
    let Some(ui) = eng.state.borrow().ui.clone() else {
        return (Value::Null, None);
    };
    let mut ui = ui.lock();
    let copy = ui.take_clipboard();
    let text = match &ui.frame().text_input {
        Some(t) => json!({
            "path": t.path,
            "rect": [t.rect.x, t.rect.y, t.rect.width, t.rect.height],
            "multiline": t.multiline,
            "selection": t.selection,
            "composing": t.composing,
        }),
        None => Value::Null,
    };
    (text, copy)
}

fn render(eng: &Engine) -> Result<bool, String> {
    if eng.render.borrow().backend.is_none() {
        return Ok(false);
    }
    let t0 = rprobe::now_ms();
    // The tick this frame shows: its notes (and a missing glyph) are this tick's.
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    let mut passes = Vec::new();
    let notes = {
        let world = eng.world()?;
        let mut rs = eng.render.borrow_mut();
        super::extract::extract(world, &mut rs);
        let rs = &mut *rs;
        if let Some(b) = rs.backend.as_mut() {
            b.set_clock(Some(rprobe::now_ms));
            b.render(&rs.scene)?;
            passes = b.pass_timings();
            if b.missing_glyphs() > 0 {
                let assets = eng.assets.borrow();
                let named = |handle: og_render::FontHandle| {
                    let at = rs.fonts.iter().position(|f| f.pages.contains(&handle))?;
                    assets
                        .fonts
                        .iter()
                        .find(|f| f.render == at)
                        .map(|f| f.id.clone())
                };
                let new = missing_glyph_notes(&rs.scene, &mut rs.missing_warned, named);
                rs.notes.extend(new);
            }
        }
        std::mem::take(&mut rs.notes)
    };
    let mut st = eng.state.borrow_mut();
    st.render_ms = rprobe::now_ms() - t0;
    super::trace::frame(eng.id, tick, t0, st.render_ms, &passes);
    st.pass_ms = passes;
    for n in notes {
        if st.ui_warned.insert(n.clone()) {
            st.log
                .push(json!({ "tick": tick, "warning": true, "text": n }));
        }
    }
    Ok(true)
}

/// The missing-glyph warnings of a frame: per font, the characters its
/// texts draw that the font lacks and no earlier frame warned of (each is
/// named once, on the tick it is first drawn: a split's minus sign at the
/// second lap is still named), with the font's asset and what its atlas
/// holds (the charset words its glyphs show, og_cook::format::charset_words).
fn missing_glyph_notes(
    scene: &og_render::RenderScene,
    warned: &mut std::collections::BTreeSet<char>,
    name: impl Fn(og_render::FontHandle) -> Option<String>,
) -> Vec<String> {
    let mut by_font: Vec<(og_render::FontHandle, Vec<char>)> = Vec::new();
    let texts = scene
        .texts
        .iter()
        .chain(scene.overlay.iter().flat_map(|o| o.texts.iter()));
    for t in texts {
        let Some(font) = scene.font(t.font) else {
            continue;
        };
        for c in t.text.chars() {
            if c.is_whitespace()
                || warned.contains(&c)
                || font.glyphs.iter().any(|g| g.codepoint == c)
            {
                continue;
            }
            warned.insert(c);
            match by_font.iter_mut().find(|(f, _)| *f == t.font) {
                Some((_, cs)) => cs.push(c),
                None => by_font.push((t.font, vec![c])),
            }
        }
    }
    by_font
        .into_iter()
        .map(|(handle, chars)| {
            let which = chars
                .iter()
                .take(8)
                .map(|c| format!("\"{c}\" U+{:04X}", *c as u32))
                .collect::<Vec<_>>()
                .join(", ");
            let more = if chars.len() > 8 {
                format!(" and {} more", chars.len() - 8)
            } else {
                String::new()
            };
            let font = scene.font(handle);
            let cps: Vec<u32> = font.map_or(Vec::new(), |f| {
                f.glyphs.iter().map(|g| g.codepoint as u32).collect()
            });
            let (words, extra) = og_cook::format::charset_words(&cps);
            let holds = match (words.is_empty(), extra) {
                (true, _) => format!("{} characters", cps.len()),
                (false, 0) => format!("charset {}", words.join(" ")),
                (false, n) => format!("charset {} and {n} more characters", words.join(" ")),
            };
            let font = name(handle).map_or("a font".to_owned(), |id| format!("font {id}"));
            let chars_arg: String = chars.iter().collect();
            format!(
                "a frame drew {which}{more}, which {font} lacks (its atlas holds {holds}): add them to its .ogt (`font {{ chars {chars_arg:?} }}`), draw a character it holds, or ship the font file (`font {{ runtime true }}`)"
            )
        })
        .collect()
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
            for w in super::e6::take_warnings(id) {
                log.push(json!({ "tick": 0, "warning": true, "text": w }));
            }
            Value::Array(log).to_string()
        }
        Err(_) => "[]".into(),
    }
}

// ── game code calls ─────────────────────────────────────────────────────────

/// Runs one registered system: publish its archetypes as views (their
/// columns are borrowed when game code takes them, `view_columns`), call
/// the dispatcher, then retire the views and release the borrows.
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
    let mut records = Vec::new();
    for (k, a) in ctx.query(0)?.enumerate() {
        let st = eng.state.borrow();
        let types = a
            .types()
            .iter()
            .filter_map(|id| st.names_by_id.get(id.index() as usize).cloned())
            .collect();
        records.push(ViewRecord {
            count: a.len() as u32,
            tick,
            entities: a.entities().as_ptr(),
            columns: Vec::new(),
            types,
            lazy: Some((k, columns.clone())),
        });
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
        held: RefCell::new(Vec::new()),
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
    // The borrows end with the system call, after the host has copied
    // writable views back (`invalidate`).
    frame.held.borrow_mut().clear();
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
    match parse(Some(value)).and_then(|v| write_resource_value(world, resource, &v)) {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

/// Writes the fields `value` names into a resource (the others keep
/// theirs): through the running system's context, which refuses a
/// resource it did not declare among its writes, or straight into the
/// world outside systems.
pub(crate) fn write_resource_value(
    world: u32,
    resource: &str,
    value: &Value,
) -> Result<(), String> {
    if let Some(f) = running(world) {
        return (|| {
            let eng = engine(world)?;
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
            let declared = eng.state.borrow().systems[f.system]
                .writes
                .iter()
                .any(|w| w == resource);
            if !declared {
                return Err(super::api::undeclared(&eng, f.system, "write", resource));
            }
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let current = ctx.resource_bytes(id).map_err(err)?;
            let bytes = layout.encode(Some(&current), Some(value))?;
            ctx.set_resource_bytes(id, &bytes).map_err(err)
        })();
    }
    (|| {
        let eng = engine(world)?;
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
        let bytes = layout.encode(Some(&current), Some(value))?;
        w.insert_resource_bytes(id, &bytes).map_err(err)
    })()
}

// ── resources as bytes (the runtime's typed handles) ────────────────────────

thread_local! {
    /// Moves whenever a world's layouts may have changed (an engine
    /// assembled, a reload swapped in): what a resource's bytes were made
    /// with travels beside them, so a handle never reads or writes them
    /// with a layout that no longer holds.
    static LAYOUT_EPOCH: Cell<u32> = const { Cell::new(1) };
}

pub(crate) fn bump_layout_epoch() {
    LAYOUT_EPOCH.with(|e| e.set(e.get().wrapping_add(1).max(1)));
}

fn layout_epoch() -> u32 {
    LAYOUT_EPOCH.with(Cell::get)
}

/// Resources whose handle is computed rather than their packed record.
fn computed_resource(name: &str) -> bool {
    name == Time::NAME || name == Rng::NAME || name == "og.Input"
}

fn kind_name(f: &super::codec::FieldLayout) -> &'static str {
    use super::codec::Kind;
    match f.kind {
        Kind::F32 => "f32",
        Kind::F64 => "f64",
        Kind::U32 => "u32",
        Kind::I32 => "i32",
        Kind::Bool => "bool",
        Kind::Vector2 => "Vector2",
        Kind::Vector3 => "Vector3",
        Kind::Quaternion => "Quaternion",
        Kind::Color => "Color",
        Kind::Matrix4 => "Matrix4",
        Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => "handle",
        Kind::Str => "str",
        Kind::Enum => "enum",
    }
}

/// `og_engine_resource_layout`.
pub fn resource_layout(world: u32, resource: &str) -> String {
    let answer = (|| {
        if computed_resource(resource) {
            return Err(format!("'{resource}' is read through its JSON value"));
        }
        let eng = engine(world)?;
        let st = eng.state.borrow();
        let layout = st
            .layouts
            .get(resource)
            .filter(|l| l.kind == RecordKind::Resource)
            .ok_or_else(|| format!("'{resource}' is not a registered resource"))?;
        let mut offset = 0;
        let fields: Vec<Value> = layout
            .fields
            .iter()
            .map(|f| {
                let size = f.ty.size();
                let mut v = json!({
                    "name": f.name, "kind": kind_name(f), "offset": offset, "size": size,
                    "readonly": f.readonly,
                });
                if !f.variants.is_empty() {
                    v["variants"] = json!(f.variants);
                }
                offset += size;
                v
            })
            .collect();
        Ok(json!({ "epoch": layout_epoch(), "size": offset, "fields": fields }))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// The id of resource `name` in the world `eng` runs (through the running
/// system's context while one runs).
fn resource_id(eng: &Engine, name: &str) -> Result<SchemaId, String> {
    if let Some(f) = running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { f.ctx() }.id_by_name(name).map_err(err);
    }
    eng.world()?.id_by_name(name).map_err(err)
}

/// `og_engine_read_resource`: the packed bytes and the layout epoch, or
/// nothing when refused.
pub fn read_resource(world: u32, resource: &str) -> Vec<u8> {
    let read = || -> Result<Vec<u8>, String> {
        if computed_resource(resource) {
            return Err(String::new());
        }
        let eng = engine(world)?;
        let id = resource_id(&eng, resource)?;
        let mut bytes = match running(world) {
            // SAFETY: the frame is published while its system runs. The
            // context refuses a resource its system did not declare.
            Some(f) => unsafe { f.ctx() }.resource_bytes(id).map_err(err)?,
            None => eng.world()?.resource_bytes(id).map_err(err)?,
        };
        bytes.extend_from_slice(&layout_epoch().to_le_bytes());
        Ok(bytes)
    };
    read().unwrap_or_default()
}

/// `og_engine_write_resource_bytes`.
pub fn write_resource_bytes(world: u32, resource: &str, bytes: &[u8]) -> String {
    let write = || -> Result<(), String> {
        let Some((record, epoch)) = bytes.split_last_chunk::<4>() else {
            return Err("the bytes carry no layout epoch".into());
        };
        if u32::from_le_bytes(*epoch) != layout_epoch() {
            return Err(format!(
                "'{resource}' was read with a layout that no longer holds (a reload or a new engine): take world.resource({resource}) again"
            ));
        }
        let eng = engine(world)?;
        let id = resource_id(&eng, resource)?;
        // Read-only fields keep the engine's value whatever the bytes say.
        let readonly: Vec<(usize, usize)> = {
            let st = eng.state.borrow();
            let layout = st
                .layouts
                .get(resource)
                .filter(|l| l.kind == RecordKind::Resource && !computed_resource(resource))
                .ok_or_else(|| format!("'{resource}' is not a registered resource"))?;
            if layout.size() != record.len() {
                return Err(format!(
                    "'{resource}' is {} bytes packed, got {}",
                    layout.size(),
                    record.len()
                ));
            }
            let mut at = 0;
            let mut out = Vec::new();
            for f in layout.fields.iter() {
                let n = f.ty.size();
                if f.readonly {
                    out.push((at, n));
                }
                at += n;
            }
            out
        };
        let patched = |current: Vec<u8>| -> Vec<u8> {
            let mut b = record.to_vec();
            for (at, n) in &readonly {
                b[*at..at + n].copy_from_slice(&current[*at..at + n]);
            }
            b
        };
        if let Some(f) = running(world) {
            let declared = eng.state.borrow().systems[f.system]
                .writes
                .iter()
                .any(|w| w == resource);
            if !declared {
                return Err(super::api::undeclared(&eng, f.system, "write", resource));
            }
            // SAFETY: the frame is published while its system runs.
            let ctx = unsafe { f.ctx() };
            let b = if readonly.is_empty() {
                record.to_vec()
            } else {
                patched(ctx.resource_bytes(id).map_err(err)?)
            };
            return ctx.set_resource_bytes(id, &b).map_err(err);
        }
        let w = eng.world()?;
        let b = if readonly.is_empty() {
            record.to_vec()
        } else {
            patched(w.resource_bytes(id).map_err(err)?)
        };
        w.insert_resource_bytes(id, &b).map_err(err)
    };
    match write() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

// ── views ───────────────────────────────────────────────────────────────────

/// The columns of `component` in view `view`, in field order, or why not.
/// A running system's view borrows them here, the first time they are
/// asked for, and holds them until the system returns.
pub fn view_columns(view: u32, component: &str) -> Result<Vec<ColumnPtr>, String> {
    HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let stale = "stale view: views last for the system call (or, outside systems, until the next structural change) they were handed out in";
        let (engine_id, rec) = t.views.get_mut(view).ok_or(stale)?;
        let cols: Vec<ColumnPtr> = rec
            .columns
            .iter()
            .filter(|(c, _)| c == component)
            .map(|(_, p)| *p)
            .collect();
        if !cols.is_empty() {
            return Ok(cols);
        }
        let wanted: Vec<QueryColumn> = rec
            .lazy
            .as_ref()
            .map(|(_, specs)| {
                specs
                    .iter()
                    .filter(|c| c.component == component)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if wanted.is_empty() {
            return Err(if rec.types.iter().any(|t| t == component) {
                format!("'{component}' is not among the reads or writes this view was made with")
            } else {
                format!("this archetype has no '{component}'")
            });
        }
        let arch = rec.lazy.as_ref().map_or(0, |(k, _)| *k);
        let frame = running(*engine_id).ok_or(stale)?;
        // SAFETY: the frame is published while its system runs; the
        // archetypes cannot move until it returns (structural changes wait).
        let ctx = unsafe { frame.ctx() };
        let a = ctx
            .query(0)
            .map_err(err)?
            .nth(arch)
            .ok_or(stale)?;
        let mut out = Vec::with_capacity(wanted.len());
        let mut held = frame.held.borrow_mut();
        for c in &wanted {
            let (ptr, guard) = if c.writable {
                let mut b = a.write_bytes(c.field).map_err(|e| taken(e, component))?;
                let p = ColumnPtr {
                    ptr: b.as_mut_ptr(),
                    len: b.len(),
                    writable: true,
                };
                // SAFETY: the borrow lives in the frame, which `run_system`
                // clears before the world's borrow for the call ends.
                (p, Held::Write(unsafe { std::mem::transmute::<std::cell::RefMut<'_, [u8]>, std::cell::RefMut<'static, [u8]>>(b) }))
            } else {
                let b = a.read_bytes(c.field).map_err(|e| taken(e, component))?;
                let p = ColumnPtr {
                    ptr: b.as_ptr() as *mut u8,
                    len: b.len(),
                    writable: false,
                };
                // SAFETY: as above.
                (p, Held::Read(unsafe { std::mem::transmute::<std::cell::Ref<'_, [u8]>, std::cell::Ref<'static, [u8]>>(b) }))
            };
            held.push(guard);
            out.push(ptr);
            rec.columns.push((component.to_string(), ptr));
        }
        Ok(out)
    })
}

/// A view's column the running code already borrows otherwise (an engine
/// call in progress holds it): the reason, with what to do.
fn taken(e: Error, component: &str) -> String {
    if e.kind() == ErrorKind::Access {
        format!(
            "the column view of '{component}' is refused: the engine holds that column for a call in progress; take the view outside that call"
        )
    } else {
        err(e)
    }
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
            for f in layout.fields.iter() {
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
            lazy: None,
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
    // `hello` only says who is speaking. The runtime asks it while creating
    // an engine (packages/engine/src/runtime.ts learnBuild), so with
    // `deferStart` it must not run GameDesc.start before the host has
    // attached its renderer and handed over its textures.
    if call.method != "hello"
        && let Err(e) = tick_boundary(&eng)
    {
        return json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string();
    }
    let answer = rprobe::with_probe(&eng, true, |p, w| rprobe::answer(p, w, call));
    let swapped = answer.as_ref().is_ok_and(|a| a.1);
    rprobe::after_reload(&eng, swapped);
    rprobe::resync_names(&eng);
    match answer {
        Ok(a) => a.0,
        Err(e) => json!({ "jsonrpc": "2.0", "id": call.id, "error": { "code": og_probe::rpc::INTERNAL_ERROR, "message": e } }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT: &str = "type project\nversion 1\n---\nname \"P\"\nentry \"src/main.ts\"\nprofile \"3d\"\ntick 30\nseed 7\nscenes {\n  main \"scenes/main.ogt\"\n  boss \"scenes/boss.ogt\"\n}\nscene boss\n";

    fn game(desc: Value) -> GameDecl {
        let i = declare_game(&desc);
        DECL.with(|d| d.borrow().games[i as usize].clone())
    }

    #[test]
    fn project_ogt_fills_what_gamedesc_leaves_unset() {
        let p = project_game(PROJECT).unwrap();
        let g = with_project(game(json!({})), &p).unwrap();
        assert_eq!((g.tick, g.seed, g.scene.as_str()), (30, 7, "boss"));
        assert_eq!(
            g.scenes.get("main").map(String::as_str),
            Some("scenes/main.ogt")
        );
        // Unset in both: the defaults.
        let none = project_game("type project\nversion 1\n---\nname \"P\"\n").unwrap();
        let g = with_project(game(json!({})), &none).unwrap();
        assert_eq!((g.tick, g.seed, g.scene.as_str()), (60, 0, "main"));
        // Set in both, alike: fine.
        let g = with_project(game(json!({ "tick": 30, "seed": 7 })), &p).unwrap();
        assert_eq!((g.tick, g.seed), (30, 7));
    }

    #[test]
    fn a_value_set_in_both_places_differently_is_refused() {
        let p = project_game(PROJECT).unwrap();
        let e = with_project(game(json!({ "seed": 1 })), &p).unwrap_err();
        assert!(
            e.contains("GameDesc.seed is 1 but project.ogt's `seed` is 7"),
            "{e}"
        );
        let e = with_project(game(json!({ "scene": "main" })), &p).unwrap_err();
        assert!(e.contains("GameDesc.scene"), "{e}");
        let e = project_game("type project\nversion 1\n---\ntick 0\n").unwrap_err();
        assert!(e.contains("project.ogt:4: `tick` takes"), "{e}");
        let e = project_game("type project\nversion 1\n---\nscenes {\n  main 3\n}\n").unwrap_err();
        assert!(e.contains("scenes.main takes"), "{e}");
    }
}
