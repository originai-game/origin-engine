//! Declarations, engines, the tick, the dispatcher and the view tables.

pub(crate) mod invariants;
pub(crate) mod lazy;

mod creation;
#[cfg_attr(feature = "publish", allow(unused_imports))]
pub(crate) use creation::assemble;
pub(crate) use creation::run_ui_registration;
pub(crate) use creation::{Assembled, extract_layouts, versioned};
pub use creation::{
    antialias_name, create, destroy, point_shadows_name, render_desc, render_options,
    set_render_backend, transparency_name, warm_renderer,
};
use creation::{boot_scene, err, run_start};
mod dispatch;
pub(crate) use dispatch::{
    Engine, Frame, Held, QueryColumn, QueryRec, SceneInstance, State, SystemEntry, Table,
    ViewRecord, before_structural, call, clear_world_queries, engine, engine_ids, invalidate,
    queued_structural, running,
};
pub use dispatch::{begin_bundle, command};
use dispatch::{run_condition, run_hook, run_system};
mod tick;
#[cfg(not(feature = "publish"))]
pub(crate) use tick::advance_probe;
pub use tick::{advance, answer_call, call_rpc, frame, take_log, take_outputs};
mod module_state;
pub(crate) use module_state::{Modules, Mount};
mod resources;
use crate::{
    author::trace, bind::agents, bind::anim, bind::i18n, bind::modules, bind::terrain,
    bind::timeline, bind::video, host::api, host::codec, ok_void, present::art,
    present::atmosphere, present::extract, present::feel, present::fx, present::interpolate,
    present::look, present::probes, present::views, world::assets, world::save, world::stream,
};
pub(crate) use resources::{
    bump_layout_epoch, layout_epoch, world_query_views, write_resource_value,
};
pub use resources::{
    component_layout, read_resource, resource_layout, view_columns, view_entities, write_resource,
    write_resource_bytes,
};

use crate::author::probe::{self as rprobe, NameOp};
use crate::host::codec::{Layout, RecordKind};
use crate::present::extract::{DrawLayouts, RenderState};
use crate::world::assets::{Assets, Fetch, Status};
use crate::{ok, parse, refuse};
use og_core::IdMap;
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
    /// 0 system, 1 command, 2 facts, 3 a run condition (answers `=true`
    /// or `=false`), 4 a hook (payload: the entity), 5 the platform pump,
    /// 6 named UI registration (world reads only).
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

impl Dispatch {
    /// The platform pump (kind 5) of `engine`: what a host's
    /// [`Hooks::pump_platform`] dispatches.
    pub fn platform_pump(engine: u32) -> Self {
        Self {
            kind: 5,
            engine,
            id: 0,
            query: 0,
            tick: 0.0,
            dt: 0.0,
            elapsed: 0.0,
            payload: String::new(),
        }
    }

    /// What the dispatcher answered, as [`Hooks::dispatch`] returns it:
    /// the text after a leading `=` is the answer, no text is no answer,
    /// and anything else is the error the game code reported.
    pub fn answer(text: String) -> Result<String, String> {
        match text.strip_prefix('=') {
            Some(payload) => Ok(payload.to_string()),
            None if text.is_empty() => Ok(String::new()),
            None => Err(text),
        }
    }
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
    /// Answer host requests between fixed ticks, outside the probe borrow.
    fn pump_platform(&self, _engine: u32) -> Result<(), String> {
        Ok(())
    }
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
    /// The native file behind a read, captured before a background audio
    /// mixer uses it. A resident host may read an immutable input generation.
    fn asset_file(&self, path: &str) -> String {
        path.to_owned()
    }
    /// A streaming group let go of its pack `pack` (it was unloaded, or the
    /// pack failed): a host that keeps fetched packs and transcoded
    /// textures drops the pack and the group's `textures` (asset ids) now,
    /// and fetches them again when the group loads again.
    fn release_stream(&self, pack: &str, textures: &[String]) {
        let _ = (pack, textures);
    }
    /// A cooked texture for the renderer, in the device's encoding when
    /// `formats` (the attached renderer's) names one: og-bundle's transcoder
    /// (native, feature `transcode`) by default, every mip level; the web
    /// host takes what its Worker transcoded with Binomial's Basis
    /// transcoder. RGBA8 without a renderer.
    fn texture(
        &self,
        texture: &og_bundle::runtime::CookedTexture,
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
    /// Starts decoding a PNG or JPEG away from the engine's thread (the web
    /// host's `createImageBitmap` in its Worker), answered later by
    /// [`Hooks::image_decode_take`] under `key`. `false`: this host cannot,
    /// and the runtime decodes on a worker thread natively (og.load.imageData).
    fn image_decode_start(&self, key: u64, bytes: &[u8]) -> bool {
        let _ = (key, bytes);
        false
    }
    /// The RGBA8 image a started decode answered, once (`Pending` until then).
    fn image_decode_take(&self, key: u64) -> Fetch<crate::world::uncooked::Decoded> {
        Fetch::Failed(format!("image decode {key} was never started"))
    }
    /// The AV1 decoder streamed videos play with here (`og.video.capabilities`):
    /// natively rav1d by default; the web host answers what its Worker found.
    fn video_decoder(&self) -> VideoDecoder {
        VideoDecoder::default()
    }
    /// On the web: a stream's AV1 units handed to the host's decoder once
    /// (`table`: offset, size and key per frame into `data`). False: this
    /// host has no decoder.
    #[cfg(target_arch = "wasm32")]
    fn video_stream(&self, stream: u64, size: (u32, u32), table: &[u32], data: &[u8]) -> bool {
        let _ = (stream, size, table, data);
        false
    }
    /// On the web: the frame player `player` of `stream` should show now
    /// (it is at `frame`), once newer than the one it shows; `None` keeps
    /// the shown one. Never waited on.
    #[cfg(target_arch = "wasm32")]
    fn video_frame(
        &self,
        stream: u64,
        player: u64,
        frame: u32,
        looped: bool,
    ) -> Option<og_render::wgpu::web_sys::VideoFrame> {
        let _ = (stream, player, frame, looped);
        None
    }
}

/// A host's AV1 decoder, as `og.video.capabilities` reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoDecoder {
    /// `rav1d`, `webcodecs`, `rav1d-wasm` or `none`.
    pub name: String,
    pub hardware: bool,
    /// The largest stream it decodes in real time at 30 fps.
    pub max_width: u32,
    pub max_height: u32,
}

impl Default for VideoDecoder {
    /// rav1d natively (8K at 30 fps on a desktop CPU); none on the web
    /// until the host says.
    fn default() -> Self {
        if cfg!(target_arch = "wasm32") {
            VideoDecoder {
                name: "none".into(),
                hardware: false,
                max_width: 0,
                max_height: 0,
            }
        } else {
            VideoDecoder {
                name: "rav1d".into(),
                hardware: false,
                max_width: 7680,
                max_height: 4320,
            }
        }
    }
}

pub use crate::present::extract::RenderBackend;

/// Handles every engine on the thread hands out, tagged with their engine:
/// interface members (`ArchetypeView.column`, `RngStream.next`) carry only
/// their object's handle.
#[derive(Default)]
pub(crate) struct Handles {
    pub views: Table<(u32, ViewRecord)>,
    /// The live views' archetype types (schema id indices), each view's a
    /// run of it (`ViewRecord::types`); cleared with the views.
    pub view_types: Vec<u32>,
    pub frame_queries: Table<(u32, QueryRec)>,
    pub world_queries: Table<(u32, QueryRec)>,
    pub streams: Table<(u32, Rng)>,
    /// og.tween.timeline's objects (the timelines are og.tween's state).
    pub timelines: Table<crate::present::feel::TimelineRef>,
    /// Spatial indexes (world.spatial), retired with the views.
    pub spatial: Table<(u32, og_ecs::SpatialHash)>,
    /// Saves open for the game's migrations (World.openSave, SaveEdit),
    /// by handle less one; a closed one's slot is reused.
    pub saves: Vec<Option<crate::world::save::Open>>,
}

// The runtime's process roots: what the host ABI holds before, across and
// between engines, never the state of one engine (that is `Engine`, its
// world and its modules' `Modules`). The game's declarations (made before
// any engine), the host's hooks, the live engines, the handles interface
// objects carry (views, queries, streams, open saves: a member names only
// its object's handle), and the call on the stack: the running system's
// frame, and whether it queued a structural change (the handles' views
// then retire as moving; a native system queues one without a frame).
// Besides these the only thread-locals are four more process roots, each
// beside its code: `host::shared` (the buffer the host and the runtime
// share), `modules` (the engine whose UI frame is open: an og.ui call
// names no world), `resources` (the layout epoch, one for the process as
// the JavaScript side keeps one) and `trace` (profiling, which starts
// before an engine exists). Nothing an engine owns is kept on the thread.
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
    /// Change filters on its query (SystemDesc.changed, added, removed).
    pub changed: Vec<String>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    /// SystemDesc.state, value and when: the state binding.
    pub state: String,
    pub value: String,
    pub when: og_ecs::StateWhen,
    /// SystemDesc.runIf's callback.
    pub condition: Option<u32>,
}

/// A hook as declared (og.defineHook).
#[derive(Clone, Debug)]
pub(crate) struct HookDecl {
    pub name: String,
    pub component: String,
    pub on: og_ecs::HookEvent,
    pub reads: Vec<Token>,
    pub writes: Vec<Token>,
    pub run: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct GameDecl {
    pub tick: u32,
    pub seed: u32,
    pub systems: Vec<u32>,
    /// GameDesc.hooks, by declaration handle.
    pub hooks: Vec<u32>,
    /// GameDesc.states.
    pub states: Vec<og_ecs::StateDesc>,
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
    /// GameDesc.ui, called after start and after a hot-reload restore.
    pub ui_setup: Option<u32>,
    /// GameDesc.look: applied before start runs; empty is none.
    pub look: String,
    /// GameDesc.saveVersion: the game's save version, written into every
    /// save; a restore runs the game's migrations from a save's version to
    /// this one (`save.rs`).
    pub save_version: u32,
    /// GameDesc.migrations' `from` versions, in declared order (the steps
    /// themselves run in TypeScript, before world.restore reaches the host).
    pub migrations: Vec<u32>,
    /// Which of tick, seed, scenes, scene, look and saveVersion GameDesc set: the others
    /// take project.ogt's ([`with_project`]), else their defaults.
    pub given: Given,
}

/// Which of GameDesc's tick, seed, scenes, scene, look and saveVersion the game set.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Given {
    pub tick: bool,
    pub seed: bool,
    pub scenes: bool,
    pub scene: bool,
    pub look: bool,
    pub save_version: bool,
}

/// project.ogt's `tick`, `seed`, `scenes`, `scene` and `look`, as the host
/// read them (CreateOptions.project): the defaults of the GameDesc fields a
/// game leaves unset.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ProjectGame {
    pub tick: Option<u32>,
    pub seed: Option<u32>,
    pub scenes: Option<BTreeMap<String, String>>,
    pub scene: Option<String>,
    /// `look "looks/<id>.ogt"`: the look whose art contract the cook reads
    /// (og-cook's `cook::look::project_look`), as an asset id.
    pub look: Option<String>,
    /// `saveVersion 3`: the game's save version (GameDesc.saveVersion).
    pub save_version: Option<u32>,
}

/// Reads `tick`, `seed`, `scenes`, `scene`, `look` and `saveVersion` from project.ogt's
/// text; a value of the wrong kind is refused naming its line. Other keys
/// are og's (`name`, `entry`, `profile`, `publish`, ...) and are not read
/// here.
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
    let look = og_bundle::look::project_look(b).map_err(|e| format!("project.ogt:{e}"))?;
    let save_version = whole(
        "saveVersion",
        1,
        u64::from(u32::MAX),
        "the game's save version, a whole number from 1",
    )?;
    Ok(ProjectGame {
        tick,
        seed,
        scenes,
        scene,
        look,
        save_version,
    })
}

/// GameDesc.migrations against GameDesc.saveVersion: each step's `from`
/// is a version before the current one, and no version has two.
pub(crate) fn check_migrations(game: &GameDecl) -> Result<(), String> {
    let mut seen = Vec::new();
    for &from in &game.migrations {
        if from == 0 || from >= game.save_version {
            return Err(format!(
                "GameDesc.migrations has a step from version {from}, and the game's saveVersion is {}: a step migrates from 1 to saveVersion - 1 (raise saveVersion when you add the step)",
                game.save_version
            ));
        }
        if seen.contains(&from) {
            return Err(format!(
                "GameDesc.migrations has two steps from version {from}: one step per version"
            ));
        }
        seen.push(from);
    }
    Ok(())
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
    // The same asset id written with or without `assets/` is one look.
    if g.look {
        game.look = og_bundle::runtime::asset_id(&game.look).to_string();
    }
    both("look", g.look, &mut game.look, &p.look)?;
    both(
        "saveVersion",
        g.save_version,
        &mut game.save_version,
        &p.save_version,
    )?;
    check_migrations(&game)?;
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
    pub hooks: Vec<HookDecl>,
    pub games: Vec<GameDecl>,
    pub commands: BTreeMap<String, CommandDecl>,
}

/// A component or resource token as the TypeScript runtime sends it
/// (`{ kind, name }`); the IDL's token types are opaque.
fn token(v: &Value) -> Option<Token> {
    Some(Token {
        kind: v["kind"].as_str()?.to_string(),
        name: v["name"].as_str()?.to_string(),
    })
}

fn tokens(list: &[crate::host::wire::OneOf<Value, Value>]) -> Vec<Token> {
    use crate::host::wire::OneOf;
    list.iter()
        .filter_map(|(OneOf::A(v) | OneOf::B(v))| token(v))
        .collect()
}

fn names(list: &[Value]) -> Vec<String> {
    list.iter().filter_map(token).map(|t| t.name).collect()
}

/// `desc` read as `T`; a refused one refuses the engine
/// (Declarations.errors) and declares at the defaults, so the handle stays
/// valid.
fn declared<T: crate::host::wire::FromJson + Default>(what: &str, name: &str, desc: &Value) -> T {
    crate::host::wire::value(desc, "desc").unwrap_or_else(|e| {
        DECL.with(|d| d.borrow_mut().errors.push(format!("{what} '{name}': {e}")));
        T::default()
    })
}

pub(crate) fn declare_system(name: String, desc: &Value, run: u32) -> u32 {
    use crate::host::wire::StateWhen;
    let d: crate::host::wire::SystemDesc = declared("og.system", &name, desc);
    let decl = SystemDecl {
        name,
        reads: tokens(&d.reads),
        writes: tokens(&d.writes),
        without: names(&d.without),
        after: d.after,
        before: d.before,
        run,
        changed: names(&d.changed),
        added: names(&d.added),
        removed: names(&d.removed),
        state: d.state,
        value: d.value,
        when: match d.when {
            StateWhen::During => og_ecs::StateWhen::During,
            StateWhen::Enter => og_ecs::StateWhen::Enter,
            StateWhen::Exit => og_ecs::StateWhen::Exit,
        },
        condition: d.run_if,
    };
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.systems.push(decl);
        d.systems.len() as u32 - 1
    })
}

pub(crate) fn declare_hook(name: String, component: String, desc: &Value, run: u32) -> u32 {
    use crate::host::wire::HookEvent;
    let d: crate::host::wire::HookDesc = declared("og.hook", &name, desc);
    let decl = HookDecl {
        name,
        component,
        on: match d.on {
            HookEvent::Add => og_ecs::HookEvent::Add,
            HookEvent::Remove => og_ecs::HookEvent::Remove,
        },
        reads: tokens(&d.reads),
        writes: tokens(&d.writes),
        run,
    };
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.hooks.push(decl);
        d.hooks.len() as u32 - 1
    })
}

pub(crate) fn declare_game(desc: &Value) -> u32 {
    // The TypeScript runtime sends forms the public type does not have: -1 for
    // a system or hook token it has no handle for (skipped), and each
    // migration as its `from` alone (the steps run in TypeScript). Those three
    // lists are read here; the rest of the desc through the generated type.
    let mut errors = Vec::new();
    let handles = |key: &str, errors: &mut Vec<String>| -> Vec<u32> {
        let mut out = Vec::new();
        for (i, h) in desc[key].as_array().into_iter().flatten().enumerate() {
            match h.as_i64() {
                Some(-1) => {}
                Some(n) if u32::try_from(n).is_ok() => out.push(n as u32),
                _ => errors.push(format!(
                    "og.game: desc.{key}[{i}]: expected a handle, got {h}"
                )),
            }
        }
        out
    };
    let systems = handles("systems", &mut errors);
    let hooks = handles("hooks", &mut errors);
    let mut migrations = Vec::new();
    for (i, m) in desc["migrations"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        match m.as_u64().or_else(|| m["from"].as_u64()).and_then(|n| u32::try_from(n).ok()) {
            Some(from) => migrations.push(from),
            None => errors.push(format!(
                "og.game: desc.migrations[{i}]: expected a version number or a SaveMigration, got {m}"
            )),
        }
    }
    let mut rest = desc.clone();
    if let Some(o) = rest.as_object_mut() {
        for key in ["systems", "hooks", "migrations"] {
            o.remove(key);
        }
    }
    // A refused desc refuses the engine (Declarations.errors); the game is
    // declared at the defaults so its handle stays valid.
    let g: crate::host::wire::GameDesc =
        crate::host::wire::value(&rest, "desc").unwrap_or_else(|e| {
            errors.push(format!("og.game: {e}"));
            Default::default()
        });
    DECL.with(|d| d.borrow_mut().errors.extend(errors));
    let game = GameDecl {
        tick: g.tick,
        seed: g.seed,
        systems,
        hooks,
        states: g
            .states
            .into_iter()
            .map(|s| og_ecs::StateDesc {
                name: s.name,
                values: s.values,
                initial: s.initial.unwrap_or_default(),
            })
            .collect(),
        resources: g
            .resources
            .iter()
            .filter_map(|r| r.as_str().map(str::to_string))
            .collect(),
        commands: g.commands.into_iter().collect(),
        command_help: g
            .command_help
            .into_iter()
            .map(|(k, c)| (k, (c.usage, c.doc)))
            .collect(),
        max_catch_up: g.max_catch_up,
        facts: g.facts,
        scene: g.scene,
        scenes: g.scenes,
        // As given: the renderer and the look read which fields it sets.
        render: desc["render"].clone(),
        start: g.start,
        ui_setup: g.ui,
        look: g.look,
        save_version: g.save_version.max(1),
        migrations,
        given: Given {
            tick: desc["tick"].is_u64(),
            seed: desc["seed"].is_u64(),
            scenes: desc["scenes"].is_object(),
            scene: desc["scene"].is_string(),
            look: desc["look"].as_str().is_some_and(|l| !l.is_empty()),
            save_version: desc["saveVersion"].is_u64(),
        },
    };
    DECL.with(|d| {
        let mut d = d.borrow_mut();
        d.games.push(game);
        d.games.len() as u32 - 1
    })
}

/// The actions the engine's modules register on a probe (`help` marks them
/// `(registered)`). Their names are reserved for the engine whatever the
/// build or the order: rprobe::MODULE_ACTIONS lists them, and a game's
/// command of one of them is refused (a test holds the list to this).
pub(crate) fn register_module_actions(probe: &mut og_probe::Probe, engine_id: u32) {
    crate::bind::agents::register_commands(probe, engine_id);
    crate::bind::timeline::register_commands(probe, engine_id);
    crate::bind::motion::register_commands(probe, engine_id);
    crate::bind::modules::docs::register_commands(probe, engine_id);
    crate::author::probe::register_render_dump(probe, engine_id);
    crate::present::probes::register_bake_dump(probe, engine_id);
    #[cfg(not(feature = "publish"))]
    crate::author::determinism::register(probe);
    #[cfg(not(feature = "publish"))]
    crate::author::inspector::register(probe, engine_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The engine's own registered actions are exactly MODULE_ACTIONS, so a
    /// module that adds one without reserving it fails here.
    #[test]
    #[cfg(not(feature = "publish"))]
    fn the_module_actions_are_the_reserved_list() {
        let mut world = World::new(&WorldDesc::default()).unwrap();
        let mut probe = og_probe::Probe::new(&mut world).unwrap();
        register_module_actions(&mut probe, 0);
        let mut have: Vec<String> = probe
            .help()
            .into_iter()
            .filter(|c| !c.core)
            .map(|c| c.name)
            .collect();
        have.sort();
        let mut want: Vec<String> = crate::author::probe::MODULE_ACTIONS
            .iter()
            .map(|s| s.to_string())
            .collect();
        want.sort();
        assert_eq!(have, want);
    }

    /// A game command named as a module action is refused as a core
    /// action's name is, whether the modules registered first or not.
    #[test]
    fn a_game_command_named_as_a_module_action_is_refused() {
        use crate::author::probe::register_one;
        for modules_first in [false, true] {
            let mut world = World::new(&WorldDesc::default()).unwrap();
            let mut probe = og_probe::Probe::new(&mut world).unwrap();
            if modules_first {
                register_module_actions(&mut probe, 0);
            }
            assert_eq!(
                register_one(&mut probe, 0, "view", "", ""),
                Err("command 'view': 'view' is a registered engine action; register under another name".to_string())
            );
            assert_eq!(
                register_one(&mut probe, 0, "dump_ai", "", ""),
                Err("command 'dump_ai': 'dump_ai' is a registered engine action; register under another name".to_string())
            );
            assert_eq!(
                register_one(&mut probe, 0, "pause", "", ""),
                Err(
                    "command 'pause': 'pause' is a core action; register under another name"
                        .to_string()
                )
            );
            assert_eq!(register_one(&mut probe, 0, "open_pause", "", ""), Ok(()));
            if !modules_first && cfg!(not(feature = "publish")) {
                // The modules registered after the game keep their own: the game's view never took it.
                register_module_actions(&mut probe, 0);
                assert!(
                    probe
                        .help()
                        .iter()
                        .any(|c| c.name == "view" && c.doc.starts_with("frame an entity"))
                );
            }
        }
    }

    const PROJECT: &str = "type project\nversion 1\n---\nname \"P\"\nentry \"src/main.ts\"\nprofile \"3d\"\ntick 30\nseed 7\nscenes {\n  main \"scenes/main.ogt\"\n  boss \"scenes/boss.ogt\"\n}\nscene boss\nsaveVersion 3\n";

    fn game(desc: Value) -> GameDecl {
        let i = declare_game(&desc);
        DECL.with(|d| d.borrow().games[i as usize].clone())
    }

    #[test]
    fn project_ogt_fills_what_gamedesc_leaves_unset() {
        let p = project_game(PROJECT).unwrap();
        let g = with_project(game(json!({})), &p).unwrap();
        assert_eq!((g.tick, g.seed, g.scene.as_str()), (30, 7, "boss"));
        assert_eq!(g.save_version, 3, "project.ogt's saveVersion");
        assert_eq!(
            g.scenes.get("main").map(String::as_str),
            Some("scenes/main.ogt")
        );
        // Unset in both: the defaults.
        let none = project_game("type project\nversion 1\n---\nname \"P\"\n").unwrap();
        let g = with_project(game(json!({})), &none).unwrap();
        assert_eq!((g.tick, g.seed, g.scene.as_str()), (60, 0, "main"));
        assert_eq!(g.save_version, 1, "a game's first save version");
        // Migrations step from 1 to saveVersion - 1, once each.
        let g = with_project(game(json!({ "migrations": [1, 2] })), &p).unwrap();
        assert_eq!(g.migrations, [1, 2]);
        let e = with_project(game(json!({ "migrations": [3] })), &p).unwrap_err();
        assert!(
            e.contains("from version 3, and the game's saveVersion is 3"),
            "{e}"
        );
        let e = with_project(game(json!({ "migrations": [1, 1] })), &p).unwrap_err();
        assert!(e.contains("two steps from version 1"), "{e}");
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
