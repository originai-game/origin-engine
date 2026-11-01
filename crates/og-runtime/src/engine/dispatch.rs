use super::*;

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

/// One scene's entities as instantiated: its asset id, the root it was
/// instantiated under (none for the boot scene) and its entities by scene
/// row, as bits.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "publish", allow(dead_code))]
pub(crate) struct SceneInstance {
    pub id: String,
    pub root: Option<u64>,
    pub entities: Vec<u64>,
}

pub(crate) struct State {
    pub layouts: BTreeMap<String, Layout>,
    /// Schema name by id index, for archetype type lists.
    pub names_by_id: Vec<String>,
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
    /// system), by bits: what `save_scene`'s spawned section writes.
    pub game_spawned: std::collections::BTreeSet<u64>,
    /// GameDesc.start, until it has run.
    pub start: Option<u32>,
    /// What GameDesc.start threw: the world is half-built, so every later
    /// advance, frame and probe call is refused with it.
    pub start_failed: Option<String>,
    /// project.ogt's game values (CreateOptions.project), for a reload's
    /// assembly, and its `look`, which turns the art contract's runtime
    /// defaults on (`super::art`).
    pub project: Option<ProjectGame>,
    /// GameDesc.look, until it has been applied (just before start).
    pub boot_look: Option<String>,
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
    /// The last frame's main-pass triangles (`ProbeTimings.triangles`).
    pub triangles: u64,
    pub warnings: Vec<String>,
    /// Log lines for `og_engine_take_log`.
    pub log: Vec<Value>,
    /// Command outputs for `og_engine_take_outputs`.
    pub outputs: Vec<og_probe::Output>,
    /// A rebuilt bundle's state, waiting for the probe's reload to swap it in.
    pub reloaded: Option<Box<Assembled>>,
    /// The physics profile (2 or 3), from the host's options.
    #[cfg_attr(feature = "publish", allow(dead_code))]
    pub profile: u32,
    /// The engine modules' handles: physics (meshes, queries) and the UI.
    pub physics: Option<og_physics::PhysicsSystem>,
    pub ui: Option<og_ui::SharedUi>,
    /// Every mesh registered with physics, replayed into a reload's physics.
    pub physics_meshes: IdMap<u64, og_physics::MeshDesc>,
    /// Every terrain's height field registered with physics, likewise.
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

// ── game code calls ─────────────────────────────────────────────────────────

/// Runs one registered system: publish its archetypes as views (their
/// columns are borrowed when game code takes them, `view_columns`), call
/// the dispatcher, then retire the views and release the borrows.
pub(super) fn run_system(
    engine_id: u32,
    index: usize,
    ctx: &mut SystemContext<'_>,
) -> og_core::Result<()> {
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
