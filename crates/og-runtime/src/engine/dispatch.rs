use super::*;

// ── engines ─────────────────────────────────────────────────────────────────

/// A registered system as the engine runs it.
pub(crate) struct SystemEntry {
    pub name: String,
    pub callback: u32,
    /// Its run condition's callback (SystemDesc.runIf).
    pub condition: Option<u32>,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub columns: Vec<QueryColumn>,
    pub matched: u32,
    pub ms: f64,
}

pub(crate) struct QueryColumn {
    /// The component's schema id index.
    pub component: u32,
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
    /// by component (schema id index).
    pub columns: Vec<(u32, ColumnPtr)>,
    /// The archetype's types as schema id indices: where they start in
    /// `Handles::view_types` and how many they are.
    pub types: (u32, u32),
    /// The rows the query's change filters pass; `None`: every row.
    pub rows: Option<Vec<u32>>,
    /// A running system's view: the system, and its archetype's place
    /// among the frame's (`Frame::archetypes`), whose declared columns it
    /// may borrow.
    pub lazy: Option<(usize, usize)>,
}

pub(crate) enum QueryRec {
    /// A running system's archetypes: this many view handles from the first.
    Frame { first: u32, len: u32 },
    /// `world.query()` outside systems: what it matches, and the views
    /// its archetypes were last described as (the first handle and how many).
    World { desc: QueryDesc, views: (u32, u32) },
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

/// One scene's entities as instantiated: its asset id, the root it was
/// instantiated under (none for the boot scene) and its entities by scene
/// row, as bits.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "publish", allow(dead_code))]
pub(crate) struct SceneInstance {
    pub id: String,
    pub root: Option<u64>,
    pub entities: Vec<u64>,
    /// Each entity's line in the scene's text (cooked: SceneLineRow), in
    /// `entities`' order; none from a publish cook.
    pub lines: Vec<Option<u32>>,
}

pub(crate) struct State {
    pub layouts: BTreeMap<String, Layout>,
    /// Schema name by id index, for archetype type lists.
    pub names_by_id: Vec<String>,
    /// Game records only other games of the process name, registered when
    /// this world's game first names one (crate::lazy::ensure).
    pub deferred: std::collections::BTreeSet<String>,
    pub resources: Vec<String>,
    pub systems: Vec<SystemEntry>,
    pub commands: BTreeMap<String, CommandDecl>,
    pub names: BTreeMap<String, u64>,
    pub name_of: IdMap<u64, String>,
    /// Name changes the probe has not seen yet.
    pub name_ops: Vec<NameOp>,
    pub rate: u32,
    pub max_catch_up: u32,
    pub accumulator_ms: f64,
    pub facts: Option<u32>,
    /// The scene GameDesc.scene names, while its bundle is still loading.
    pub boot_scene: Option<String>,
    /// The asset id of the scene GameDesc.scene spawned (`save_scene`
    /// writes its source back first, with the resources and the spawned
    /// section).
    pub scene_source: Option<String>,
    /// Every scene instantiated, in order (the boot scene's, those of
    /// og.world.instantiate): `save_scene` writes each back to its source.
    pub scene_instances: Vec<SceneInstance>,
    /// Entities the game spawned (world.spawn, between ticks or in a
    /// system), by bits: what `save_scene`'s spawned section writes. Each
    /// with the line that spawned it (SpawnDesc.source), which the probe
    /// answers as its source; none in a publish build.
    pub game_spawned: BTreeMap<u64, Option<std::sync::Arc<str>>>,
    /// GameDesc.start, until it has run.
    pub start: Option<u32>,
    /// Pending registration, including an absent GameDesc.ui. None means it ran.
    pub ui_setup: Option<Option<u32>>,
    /// What GameDesc.start threw: the world is half-built, so every later
    /// advance, frame and probe call is refused with it.
    pub start_failed: Option<String>,
    /// project.ogt's game values (CreateOptions.project), for a reload's
    /// assembly, and its `look`, which turns the art contract's runtime
    /// defaults on (`super::art`).
    pub project: Option<ProjectGame>,
    /// GameDesc.look, until it has been applied (just before start).
    pub boot_look: Option<String>,
    /// The game's save version (GameDesc.saveVersion, project.ogt's
    /// `saveVersion`): written into every save.
    pub save_version: u32,
    /// The last UiDesc given to og.ui.configure, which applying a look's
    /// style keeps.
    pub ui_desc: Value,
    pub step_ms: f64,
    pub render_ms: f64,
    /// The last frame's render-graph passes and their encoding (start, ms).
    pub pass_ms: Vec<og_render::PassTiming>,
    /// The GPU milliseconds per pass of the newest frame the GPU's
    /// timestamps were read back for (`ProbeTimings.gpuPasses`).
    pub gpu_ms: Vec<(String, f64)>,
    /// What the last frame submitted: its triangles are
    /// `ProbeTimings.triangles`, the rest `dump_render`'s.
    pub frame: og_render::FrameStats,
    pub warnings: Vec<String>,
    /// Log lines for `og_engine_take_log`.
    pub log: Vec<Value>,
    /// Command outputs for `og_engine_take_outputs`.
    pub outputs: Vec<og_probe::Output>,
    /// Accepted reflected inputs, drained by input.takeIntents.
    pub recorded_intents: Vec<Value>,
    /// A rebuilt bundle's state, waiting for the probe's reload to swap it in.
    pub reloaded: Option<Box<Assembled>>,
    /// The physics profile (2 or 3), from the host's options.
    #[cfg_attr(feature = "publish", allow(dead_code))]
    pub profile: u32,
    /// The engine modules' handles: physics (meshes, queries) and the UI.
    pub physics: Option<og_physics::PhysicsSystem>,
    pub ui: Option<og_ui::SharedUi>,
    /// The world's gameplay actions (og.input.defineAction and friends).
    pub actions: Option<og_input::SharedActions>,
    /// Every mesh registered with physics, replayed into a reload's physics.
    pub physics_meshes: IdMap<u64, og_physics::MeshDesc>,
    /// The triangle-mesh colliders the cook baked, by Geometry handle
    /// (`modules::set_baked_trimesh`), given again to a rebuilt physics.
    #[cfg(feature = "physics3d")]
    pub physics_baked: IdMap<u64, Vec<u8>>,
    /// Every terrain's height field registered with physics, likewise.
    #[cfg_attr(all(feature = "publish", feature = "profile2d"), allow(dead_code))]
    pub physics_heightfields: IdMap<u64, og_physics::HeightfieldDesc>,
    /// UI warnings already logged, so a repeated one is logged once.
    pub ui_warned: BTreeSet<String>,
    /// The host mirrors the UI's accessibility tree (create option `a11y`,
    /// the web Worker): frame answers carry the tree when it changed.
    pub a11y_mirror: bool,
    /// The digest of the accessibility tree the last frame answer carried
    /// (`A11yTree::digest`; 0: none sent yet), so the mirror is sent the
    /// tree only when it changes.
    pub a11y_digest: u64,
    /// The tree the last frame answer carried and the UI's semantic frame
    /// then, which live regions announce against (`A11yTree::announcements`).
    pub a11y_sent: Option<(u64, og_ui::A11yTree)>,
    /// Cursor images decoded from their textures, by source
    /// ([`crate::ui_cursor`]).
    pub cursor_images: crate::ui_cursor::Kept,
    /// The cursor the last frame answer carried (its keyword and image id),
    /// so its image is sent only when it changes.
    pub cursor_sent: Option<(&'static str, u64)>,
    /// Undeclared accesses already logged (`system access name`), so each is
    /// logged once per system (`api::undeclared`), and the calls handed a
    /// bare index for an entity (`api::live`), once per system and call.
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
    pub(super) world: UnsafeCell<World>,
    pub stepping: Cell<bool>,
    pub state: RefCell<State>,
    /// og-probe over this engine's world: the command channel and protocol 5.
    pub probe: RefCell<og_probe::Probe>,
    /// The renderer side, shared with the probe's capture hook.
    pub render: Rc<RefCell<RenderState>>,
    pub assets: RefCell<Assets>,
    /// The run's assertions (the probe's, shared): the game's og.assert
    /// calls reach it while a system runs, when the probe is busy.
    pub assertions: Rc<RefCell<og_probe::assertions::Assertions>>,
    /// Each runtime module's state on this engine.
    pub modules: super::Modules,
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
    /// The system's matched archetypes, in its views' order. The lifetime
    /// is the world's for the system call, as `held`'s.
    pub archetypes: Vec<og_ecs::ArchetypeView<'static>>,
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
        t.view_types.clear();
        t.frame_queries.clear();
        t.spatial.clear();
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

// ── game code calls ─────────────────────────────────────────────────────────

/// Where a system call's query description starts in the shared buffer, in
/// words: after the four doubles of the time scale.
pub(crate) const FRAME_QUERY_WORD: usize = 8;

/// Runs one registered system: publish its archetypes as views (their
/// columns are borrowed when game code takes them, `view_columns`), write
/// the call's frame description (the time scale and the archetypes) to the
/// shared buffer, call the dispatcher, then retire the views and release
/// the borrows.
pub(super) fn run_system(
    engine_id: u32,
    index: usize,
    ctx: &mut SystemContext<'_>,
) -> og_core::Result<()> {
    let fail = |m: String| Error::new(ErrorKind::Invalid, m);
    let eng = engine(engine_id).map_err(fail)?;
    let callback = eng.state.borrow().systems[index].callback;
    let time = ctx.time();
    let tick = time.tick as u32;
    let archetypes: Vec<og_ecs::ArchetypeView<'_>> = ctx.query(0)?.collect();
    let (query, matched) = HANDLES.with(|t| {
        let mut t = t.borrow_mut();
        let t = &mut *t;
        let first = t.views.base.wrapping_add(t.views.items.len() as u32);
        let mut matched = 0;
        for (k, a) in archetypes.iter().enumerate() {
            let start = t.view_types.len() as u32;
            t.view_types.extend(a.types().iter().map(|id| id.index()));
            matched += a.len() as u32;
            t.views.push((
                engine_id,
                ViewRecord {
                    count: a.len() as u32,
                    tick,
                    entities: a.entities().as_ptr(),
                    columns: Vec::new(),
                    types: (start, a.types().len() as u32),
                    rows: a.filtered().then(|| a.rows()),
                    lazy: Some((index, k)),
                },
            ));
        }
        let rec = QueryRec::Frame {
            first,
            len: archetypes.len() as u32,
        };
        (t.frame_queries.push((engine_id, rec)), matched)
    });
    // What the dispatcher reads first: the time scale, then the archetypes.
    let scale = ctx.time_scale();
    crate::shared::with(|b| {
        super::feel::put_time(
            b,
            scale.scale,
            scale.hit_stop,
            scale.dt(&time),
            scale.elapsed,
        );
    });
    api::describe_query(query, 0, FRAME_QUERY_WORD);
    let frame = Frame {
        engine: engine_id,
        system: index,
        ctx: ctx as *mut SystemContext<'_> as *mut (),
        // SAFETY: the views live in the frame, which this call drops before
        // the world's borrow for the call ends.
        archetypes: unsafe {
            std::mem::transmute::<Vec<og_ecs::ArchetypeView<'_>>, Vec<og_ecs::ArchetypeView<'static>>>(
                archetypes,
            )
        },
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
            payload: String::new(),
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

/// Calls game code with `ctx` published as the running frame of entry
/// `index` (a run condition, kind 3, under its system's entry; a hook,
/// kind 4, under its own), as [`run_system`] does without views.
fn run_frame(
    engine_id: u32,
    index: usize,
    ctx: &mut SystemContext<'_>,
    kind: u32,
    payload: String,
) -> og_core::Result<String> {
    let fail = |m: String| Error::new(ErrorKind::Invalid, m);
    let eng = engine(engine_id).map_err(fail)?;
    let callback = match kind {
        3 => eng.state.borrow().systems[index].condition,
        _ => Some(eng.state.borrow().systems[index].callback),
    }
    .ok_or_else(|| fail("no callback".into()))?;
    let time = ctx.time();
    let frame = Frame {
        engine: engine_id,
        system: index,
        ctx: ctx as *mut SystemContext<'_> as *mut (),
        archetypes: Vec::new(),
        held: RefCell::new(Vec::new()),
    };
    let prev = FRAME.with(|f| f.replace(&frame));
    let was_stepping = eng.stepping.replace(true);
    let result = match hooks() {
        Some(h) => h.dispatch(&Dispatch {
            kind,
            engine: engine_id,
            id: callback,
            query: 0,
            tick: time.tick as f64,
            dt: f64::from(time.dt),
            elapsed: time.elapsed,
            payload,
        }),
        None => Err("no dispatcher is installed (og_host_set_dispatcher)".into()),
    };
    FRAME.with(|f| f.set(prev));
    eng.stepping.set(was_stepping);
    invalidate(QUEUED.with(|q| q.replace(false)));
    frame.held.borrow_mut().clear();
    result.map_err(fail)
}

/// A system's run condition (SystemDesc.runIf): whether it runs.
pub(super) fn run_condition(
    engine_id: u32,
    index: usize,
    ctx: &mut SystemContext<'_>,
) -> og_core::Result<bool> {
    let answer = run_frame(engine_id, index, ctx, 3, String::new())?;
    Ok(answer == "true")
}

/// A hook (og.defineHook) on entity `e`.
pub(super) fn run_hook(
    engine_id: u32,
    index: usize,
    ctx: &mut SystemContext<'_>,
    e: Entity,
) -> og_core::Result<()> {
    run_frame(engine_id, index, ctx, 4, e.to_bits().to_string()).map(|_| ())
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
    let args = match parse(Some(args)) {
        Ok(Value::Array(a)) => Value::Array(a),
        Ok(Value::Null) => json!([]),
        _ => return refuse("command arguments are a JSON list of strings"),
    };
    let handler = eng.state.borrow().commands.get(action).map(|c| c.handler);
    let Some(handler) = handler else {
        // Not the game's: a module's action registered with the probe
        // (locale, view, cover), run on the world now.
        let words: Option<Vec<String>> = args
            .as_array()
            .and_then(|a| a.iter().map(|v| v.as_str().map(str::to_owned)).collect());
        let Some(words) = words else {
            return refuse("command arguments are a JSON list of strings");
        };
        let world = match eng.world() {
            Ok(w) => w,
            Err(e) => return refuse(e),
        };
        let result = eng.probe.borrow_mut().act(world, action, &words);
        return match result {
            Some(Ok(())) => super::ok_void(),
            Some(Err(e)) => refuse(format!("command '{action}': {e}")),
            None => {
                let mut known: Vec<String> = eng.state.borrow().commands.keys().cloned().collect();
                known.extend(eng.probe.borrow().registered());
                og_core::sort::sort(&mut known);
                refuse(format!(
                    "no command action '{action}'; the game and its modules register {known:?}"
                ))
            }
        };
    };
    match call(&eng, 1, handler, args.to_string()) {
        Ok(_) => super::ok_void(),
        Err(e) => refuse(format!("command '{action}': {e}")),
    }
}
