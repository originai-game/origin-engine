use super::*;

use og_api::host::{DISPATCH_COMMAND, DISPATCH_UI};

mod assemble;
mod render;

pub(crate) use assemble::*;
pub use render::*;

// ── creation ────────────────────────────────────────────────────────────────

pub(super) fn err(e: Error) -> String {
    e.reason().to_string()
}

/// `og_engine_create`.
pub fn create(game: u32, options: &str) -> String {
    match create_inner(game, options) {
        Ok(id) => ok(json!(id)),
        Err(e) => refuse(e),
    }
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
            rect: get("og.RectLight"),
            fx: super::fx::FxLayouts::from(get),
            atmo: super::atmosphere::AtmoLayouts::from(get, st.physics.clone()),
            environment: get("og.Environment"),
            terrain: get("og.Terrain"),
            scatter: get("og.TerrainScatter"),
            grass: get("og.Grass"),
            stamp: get("og.TerrainStamp"),
            grass_pusher: get("og.GrassPusher"),
            scatter_clearing: get("og.ScatterClearing"),
            grass_type: get("og.GrassType"),
            collider: get("og.Collider"),
            force_field: get("og.ForceField"),
            lines: get("og.Lines"),
            points: get("og.Points"),
            probe: get("og.ReflectionProbe"),
            light_volume: get("og.LightVolume"),
            realtime_volume: get(crate::present::probes::REALTIME_LIGHT_VOLUME),
            volume: get("og.Volume"),
            sight: get(crate::bind::sight::MAP),
            interpolation: get("og.Interpolation"),
            origin: get(crate::world::origin::RESOURCE),
            views: super::views::ViewLayouts::from(get),
            physics: st.physics.clone(),
            engine: eng.id,
        },
    )
}

fn create_inner(game: u32, options: &str) -> Result<u32, String> {
    let options = parse(Some(options))?;
    let engine_id = ENGINES.with(|e| e.borrow().len() as u32);
    let seed = options["seed"].as_u64().map(|s| s as u32);
    // project.ogt's tick, seed, scenes and scene: what GameDesc leaves unset.
    let project = match options["project"].as_str() {
        Some(text) => Some(project_game(text)?),
        None => None,
    };
    // The profile is the project's creation form's; an engine without a
    // project is 3D.
    let profile = if project.as_ref().is_some_and(|p| p.two_d) {
        og_physics::PROFILE_2D
    } else {
        og_physics::PROFILE_3D
    };
    let modules = super::Modules::default();
    let mount = super::Mount {
        id: engine_id,
        modules: &modules,
    };
    let mut a = assemble(game, seed, mount, profile, project.as_ref(), None)?;
    let probe = a.probe.take().expect("assemble makes a probe");
    let boot_flow = project
        .as_ref()
        .and_then(|p| p.flow.as_ref())
        .map(|f| f.dialogue.clone());
    let state = State {
        layouts: a.layouts,
        names_by_id: a.names_by_id,
        deferred: a.deferred,
        resources: a.resources,
        systems: a.systems,
        commands: a.commands,
        names: BTreeMap::new(),
        name_of: IdMap::new(),
        name_ops: Vec::new(),
        held_names: BTreeSet::new(),
        rate: a.game.tick,
        max_catch_up_ticks: a.game.max_catch_up_ticks.max(1),
        accumulator_ms: 0.0,
        facts: a.game.facts,
        scenes: a.game.scenes.clone(),
        boot_scene: None,
        scene_source: None,
        scene_instances: Vec::new(),
        game_spawned: Default::default(),
        start: a.game.start,
        ui_setup: Some(a.game.ui_setup),
        start_failed: None,
        project,
        boot_look: (!a.game.look.is_empty()).then(|| a.game.look.clone()),
        boot_flow,
        save_version: a.game.save_version,
        ui_desc: Value::Null,
        ui_font_roles: Default::default(),
        step_ms: 0.0,
        render_ms: 0.0,
        pass_ms: Vec::new(),
        gpu_ms: Vec::new(),
        gpu_frame_ms: None,
        frame: Default::default(),
        // A GameDesc mistake is logged at once (the others answer the call that meets them).
        log: a
            .warnings
            .iter()
            .filter(|w| w.starts_with("GameDesc."))
            .map(|w| json!({ "tick": 0, "warning": true, "text": w }))
            .collect(),
        warnings: a.warnings,
        outputs: Vec::new(),
        recorded_intents: Vec::new(),
        reloaded: None,
        profile,
        physics: Some(a.mounted.physics.clone()),
        actions: Some(a.mounted.actions.clone()),
        physics_meshes: IdMap::new(),
        physics_baked: IdMap::new(),
        physics_heightfields: IdMap::new(),
        ui_warned: BTreeSet::new(),
        ui_answers: Default::default(),
        ui_frame: Default::default(),
        a11y_mirror: options["a11y"].as_bool().unwrap_or(false),
        cursor_sent: None,
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
    render.borrow_mut().engine = engine_id;
    let engine = Rc::new(Engine {
        id: engine_id,
        world: UnsafeCell::new(a.world),
        stepping: Cell::new(false),
        time_read: Default::default(),
        state: RefCell::new(state),
        assertions: probe.assertions(),
        probe: RefCell::new(probe),
        render,
        assets: RefCell::new(Assets {
            root: options["root"].as_str().unwrap_or_default().to_string(),
            content: options["content"].as_str().unwrap_or_default().to_string(),
            content_later: options["contentLater"].as_bool().unwrap_or(false),
            ..Assets::default()
        }),
        modules,
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
        super::agents::commit(&engine, probe.inbox());
        super::timeline::commit(&engine, probe.inbox());
        register_module_actions(&mut probe, engine_id);
    }
    // The host's locale (og run --locale, the browser's language).
    super::i18n::request(&engine, options["locale"].as_str().unwrap_or(""));
    ENGINES.with(|e| e.borrow_mut().push(Some(engine.clone())));
    #[cfg(not(feature = "publish"))]
    rprobe::set_reloader(&engine);

    // The game's bundle and its first scene, before the first tick.
    if !engine.assets.borrow().root.is_empty() {
        let i = engine.assets.borrow_mut().open_bundle("cooked");
        engine.assets.borrow_mut().default_bundle = Some(i);
    }
    // With `deferStart` the scene waits for the first tick boundary too: the
    // web host transcodes the bundle's Basis textures only once its renderer
    // is attached, and a scene spawned before that finds none of them.
    let defer = options["deferStart"].as_bool().unwrap_or(false);
    if let Some(path) = a.game.scenes.get(&a.game.scene) {
        engine.state.borrow_mut().boot_scene = Some(path.clone());
        if !defer {
            boot_scene(&engine);
        }
    }
    // GameDesc.look's antialias, before the host attaches its renderer.
    if !a.game.look.is_empty() {
        super::look::boot_antialias(&engine, &a.game.look);
    }
    // `deferStart`: the host attaches its renderer and reports the viewport
    // first; start then runs at the first tick boundary.
    if !defer {
        run_start(&engine)?;
    }
    Ok(engine_id)
}

/// GameDesc.start, once, when the first scene is in (or there is none).
pub(super) fn run_start(eng: &Engine) -> Result<(), String> {
    if eng.state.borrow().boot_scene.is_some() {
        return Ok(());
    }
    // The locale's font-pack faces, which the web fetches after the boot
    // (docs/ui.md §3.4): start waits for them as it waits for the scene, so
    // an og.load.font of one in start finds it, and the first frame needs
    // them anyway.
    if eng.state.borrow().start.is_some() && crate::bind::i18n::faces_pending(eng) {
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
    // project.ogt's `flow`: started before GameDesc.start, which may read
    // it (og.flow.current) or pause it behind a title screen.
    let flow = eng.state.borrow_mut().boot_flow.take();
    if let Some(dialogue) = flow
        && let Err(e) = start_flow(eng, &dialogue)
    {
        let text = format!("project.ogt's `flow`: {e}");
        eng.state.borrow_mut().start_failed = Some(text.clone());
        return Err(text);
    }
    let start = eng.state.borrow_mut().start.take();
    if let Some(start) = start {
        before_structural(eng);
        if let Err(e) = call(eng, DISPATCH_COMMAND, start, "[]".into()) {
            let text =
                format!("GameDesc.start threw, so the world is half-built and does not run: {e}");
            eng.state.borrow_mut().start_failed = Some(text.clone());
            return Err(text);
        }
    }
    let ui_setup = eng.state.borrow_mut().ui_setup.take();
    if let Some(callback) = ui_setup {
        run_ui_registration(eng, callback)?;
    }
    Ok(())
}

/// Loads, prepares and starts the flow project.ogt declares (its
/// dialogue's asset id), as a game's og.story.load, og.flow.load and
/// og.flow.start would.
fn start_flow(eng: &Engine, dialogue: &str) -> Result<(), String> {
    let story = crate::bind::story::og_story_load(eng.id, dialogue.to_string())?;
    crate::bind::flow::og_flow_load(eng.id, story.name.clone())?;
    crate::bind::flow::start_declared(eng.id, &story.name)
}

pub(crate) fn run_ui_registration(eng: &Engine, callback: Option<u32>) -> Result<(), String> {
    if callback.is_none() && hooks().is_none() {
        return Ok(());
    }
    let before = eng.world()?.world_hash();
    // `flow`: the TypeScript runtime mounts the built-in FlowScreen first,
    // unless project.ogt's `flow` block says the game draws it.
    let screen = eng
        .state
        .borrow()
        .project
        .as_ref()
        .and_then(|p| p.flow.as_ref())
        .is_some_and(|f| !f.game_screen);
    // An absent callback on reload is still a registration pass: it disposes
    // the presentation IDs removed with the old GameDesc.ui.
    let result = call(
        eng,
        DISPATCH_UI,
        callback.unwrap_or(u32::MAX),
        if screen { "flow".into() } else { String::new() },
    );
    let after = eng.world()?.world_hash();
    let result = if before == after {
        result.map(|_| ())
    } else {
        Err("registration changed world_hash; UI setup may only read gameplay state".into())
    };
    result.map_err(|reason| {
        let reason = format!("GameDesc.ui: {reason}");
        eng.state.borrow_mut().start_failed = Some(reason.clone());
        reason
    })
}

/// Spawns GameDesc.scene once its bundle is ready; a failed bundle is a
/// warning and the world starts without it.
pub(super) fn boot_scene(eng: &Engine) {
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
    ENGINES.with(|e| {
        if let Some(slot) = e.borrow_mut().get_mut(id as usize) {
            *slot = None;
        }
    });
    true
}
