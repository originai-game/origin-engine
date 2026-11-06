use super::*;

// ── creation ────────────────────────────────────────────────────────────────

/// Declaration order, then `after`/`before`: among the systems whose
/// predecessors have all been placed, the earliest declared goes next.
fn order(systems: &[SystemDecl]) -> Result<Vec<usize>, String> {
    let index: BTreeMap<&str, usize> = og_core::sort::map(
        systems
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.as_str(), i)),
    );
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

/// A world assembled from a game's declarations, with the runtime's view of
/// it. Engine creation and hot reload both build one.
pub(crate) struct Assembled {
    pub world: World,
    pub probe: Option<og_probe::Probe>,
    pub layouts: BTreeMap<String, Layout>,
    pub names_by_id: Vec<String>,
    /// Game records only other games name, left unregistered until this
    /// game names one (super::super::lazy::ensure).
    pub deferred: BTreeSet<String>,
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
    "og.PointerLock",
];

/// An engine record's schema at the version its module registers, so the
/// module adopts the IDL's record: a record whose fields changed since
/// saves held it is at a later version (og-audio's og.AudioSource and
/// og.AudioBusFx, version 2: the bus graph and the bus effects;
/// og-physics's og.RigidBody and og.Physics, version 2: sleep; og.Joint,
/// version 2: breaking; og.Vehicle, version 2: the drivetrain; og.Cloth,
/// og.SoftBody, og.Garment and og.Fluid3D, version 2: simulation; og.Skeleton2D,
/// version 2: a second blended clip; the runtime's
/// og.VideoPlayer, version 2: an f64 time; og.Wind, version 2: the
/// simulated field's turbulence and gust size).
pub(crate) fn versioned(mut schema: og_core::Schema) -> og_core::Schema {
    use og_core::Reflect;
    schema.version = match schema.name.as_str() {
        "og.AudioSource" => og_audio::AudioSource::schema().version,
        "og.AudioBusFx" => og_audio::AudioBusFx::schema().version,
        "og.RigidBody" => og_physics::RigidBody::schema().version,
        "og.Physics" => og_physics::Physics::schema().version,
        "og.Joint" => og_physics::Joint::schema().version,
        "og.Vehicle" => og_physics::Vehicle::schema().version,
        // R9: 64 blendshape weights (`influences`).
        "og.Blendshapes" => og_anim::Blendshapes::schema().version,
        // The 2D graph: Skeleton2D's blendClip, blendTime and blend.
        "og.Skeleton2D" => og_anim::Skeleton2D::schema().version,
        // simulation (gameplay or visual).
        "og.Cloth" => og_physics::Cloth::schema().version,
        "og.SoftBody" => og_physics::SoftBody::schema().version,
        "og.Garment" => og_physics::Garment::schema().version,
        "og.Fluid3D" => og_physics::Fluid3D::schema().version,
        crate::video::PLAYER => crate::video::PLAYER_VERSION,
        // R11: Light2D's kind, cone, height and layers; Occluder2D's
        // polygon and layers; Lighting2D's shadowAngles.
        "og.Light2D" | "og.Occluder2D" | "og.Lighting2D" => 2,
        // G2: turbulence and gustSize, the simulated field's.
        "og.Wind" => 2,
        _ => return schema,
    };
    schema
}

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
    let (decl, others, records, errors, all_systems, all_hooks, commands) = DECL.with(|d| {
        let d = d.borrow();
        (
            d.games.get(game as usize).cloned(),
            d.games
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != game as usize)
                .map(|(_, g)| g.clone())
                .collect::<Vec<_>>(),
            d.records.clone(),
            d.errors.clone(),
            d.systems.clone(),
            d.hooks.clone(),
            d.commands.clone(),
        )
    });
    let game = decl.ok_or_else(|| format!("no game {game}: declare it with og.game first"))?;
    let game = match project {
        Some(p) => with_project(game, p)?,
        None => {
            super::check_migrations(&game)?;
            game
        }
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
            || decl.schema == crate::vfx::EVENT
        {
            continue;
        }
        // A lazy record (super::lazy): its layout now, registered with the
        // world when a game first uses it, so a world without it keeps its
        // type table and hash.
        if crate::lazy::is_lazy(decl.schema) {
            match Layout::from_decl(decl) {
                Ok(l) => {
                    layouts.insert(l.schema.clone(), l);
                }
                Err(e) => warnings.push(format!("{} not registered: {e}", decl.schema)),
            }
            continue;
        }
        match Layout::from_decl(decl).and_then(|l| {
            world.register_schema(versioned(l.schema())).map_err(err)?;
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
    // og.VfxEvent (its name is a str64): its layout from its Rust type,
    // registered lazily (super::super::lazy) like og.Vfx.
    {
        use og_core::Reflect;
        let l = Layout::from_schema(&crate::vfx::VfxEvent::schema(), RecordKind::Event);
        layouts.insert(l.schema.clone(), l);
    }
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
    // their defaults. A record only other games of the process name (a
    // second game's resource, declared beside this one in a test run) is
    // not this world's: it is registered when this game first names it
    // (super::super::lazy::ensure), so one game's declarations never reach
    // another's type table, snapshot or hash.
    let ours = named_by(&game, &all_systems, &all_hooks);
    let theirs: BTreeSet<String> = others
        .iter()
        .flat_map(|g| named_by(g, &all_systems, &all_hooks))
        .filter(|n| !ours.contains(n))
        .collect();
    let mut deferred = BTreeSet::new();
    for layout in &records {
        if theirs.contains(&layout.schema) {
            deferred.insert(layout.schema.clone());
            layouts.insert(layout.schema.clone(), layout.clone());
            continue;
        }
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
    resources.push("og.PointerLock".to_string());
    // og-input's records (og.Input, its further resources and the input
    // events) are read through the layout og-input registered, so a value
    // carries every field the schema declares.
    for decl in og_api::host::RECORDS {
        if !REGISTERED_BY_MODULES.contains(&decl.schema) {
            continue;
        }
        let kind = match decl.kind {
            "event" => RecordKind::Event,
            "resource" => RecordKind::Resource,
            _ => continue,
        };
        let id = world.id_by_name(decl.schema).map_err(err)?;
        let mut layout = Layout::from_schema(world.schema(id).map_err(err)?, kind);
        // A resource's fields are as writable as the IDL says (og.PointerLock's
        // `wanted` is the game's); the host's input stays read only.
        if kind == RecordKind::Resource {
            for f in std::sync::Arc::make_mut(&mut layout.fields) {
                f.readonly = decl
                    .fields
                    .iter()
                    .find(|d| d.name == f.name)
                    .is_none_or(|d| d.readonly);
            }
        }
        layouts.insert(layout.schema.clone(), layout);
    }

    // The modules' systems before the game's (physics' collider recording,
    // the UI's begin), then the game's, then the modules' after them.
    let ui = super::modules::new_ui(game.tick, previous.as_ref().map(|p| p.0));
    let (physics, actions) = super::modules::mount_front(&mut world, engine_id, profile, &ui)?;
    let record_names: Vec<String> = layouts
        .keys()
        .filter(|n| world.id_by_name(n).is_ok())
        .cloned()
        .collect();
    super::terrain::mount(&mut world, engine_id)?;
    super::e6::mount_front(&mut world, engine_id, profile, &physics, &record_names)?;

    // State machines, before the systems bound to them.
    for state in &game.states {
        world.add_state(state.clone()).map_err(err)?;
    }
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
    // Lazy records a game system or hook names: registered before the
    // systems that declare them, their modules granted them once mounted.
    let lazy_named: Vec<&'static str> = crate::lazy::LAZY
        .iter()
        .map(|l| l.name)
        .filter(|&name| {
            systems_decl.iter().any(|s| {
                s.reads.iter().chain(&s.writes).any(|t| t.name == name)
                    || s.without
                        .iter()
                        .chain(&s.changed)
                        .chain(&s.added)
                        .chain(&s.removed)
                        .any(|n| n == name)
            }) || game.hooks.iter().any(|&h| {
                all_hooks.get(h as usize).is_some_and(|h| {
                    h.component == name || h.reads.iter().chain(&h.writes).any(|t| t.name == name)
                })
            })
        })
        .flat_map(crate::lazy::with_partners)
        .collect();
    // In LAZY's order, once each (a partner joins at its own place).
    let lazy_named: Vec<&'static str> = crate::lazy::LAZY
        .iter()
        .map(|l| l.name)
        .filter(|n| lazy_named.contains(n))
        .collect();
    for name in &lazy_named {
        if let Some(l) = layouts.get(*name) {
            let id = world.register_schema(versioned(l.schema())).map_err(err)?;
            if l.kind == RecordKind::Resource {
                world
                    .insert_resource_bytes(id, &l.encode(None, None)?)
                    .map_err(err)?;
            }
        }
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
        // Change filters: changed and added components are read (and so
        // required); removed ones are only matched against.
        for name in s.changed.iter().chain(&s.added) {
            if !q.write.contains(name) && !q.read.contains(name) {
                q.read.push(name.clone());
            }
        }
        q.changed = s.changed.clone();
        q.added = s.added.clone();
        q.removed = s.removed.clone();
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
        let mut reads = names(&s.reads);
        for name in s.changed.iter().chain(&s.added).chain(&s.removed) {
            if !reads.contains(name) {
                reads.push(name.clone());
            }
        }
        let desc = SystemDesc {
            name: s.name.clone(),
            reads: reads.clone(),
            writes: names(&s.writes),
            queries: vec![q],
            state: (!s.state.is_empty()).then(|| og_ecs::StateRun {
                state: s.state.clone(),
                value: s.value.clone(),
                when: s.when,
            }),
        };
        let index = systems.len();
        let added = match s.condition {
            Some(_) => world.add_system_if(
                desc,
                move |ctx| run_condition(engine_id, index, ctx),
                move |ctx| run_system(engine_id, index, ctx),
            ),
            None => world.add_system(desc, move |ctx| run_system(engine_id, index, ctx)),
        };
        added.map_err(err)?;
        systems.push(SystemEntry {
            name: s.name.clone(),
            callback: s.run,
            condition: s.condition,
            reads,
            writes: names(&s.writes),
            columns,
            matched: 0,
            ms: 0.0,
        });
    }
    // Hooks, in GameDesc.hooks' order: their entries follow the systems'.
    for &h in &game.hooks {
        let hook = all_hooks
            .get(h as usize)
            .cloned()
            .ok_or_else(|| format!("the game lists hook {h}, which was never declared"))?;
        let names = |ts: &[Token]| ts.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        let index = systems.len();
        world
            .add_hook(
                og_ecs::HookDesc {
                    name: hook.name.clone(),
                    component: hook.component.clone(),
                    on: hook.on,
                    reads: names(&hook.reads),
                    writes: names(&hook.writes),
                },
                move |ctx, e| run_hook(engine_id, index, ctx, e),
            )
            .map_err(err)?;
        systems.push(SystemEntry {
            name: hook.name.clone(),
            callback: hook.run,
            condition: None,
            reads: names(&hook.reads),
            writes: names(&hook.writes),
            columns: Vec::new(),
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
    // Registered ones only: og.PrefabLink's layout is known before the
    // world registers it (crate::prefab).
    let components: Vec<String> = layouts
        .values()
        .filter(|l| l.kind == RecordKind::Component && world.id_by_name(&l.schema).is_ok())
        .map(|l| l.schema.clone())
        .collect();
    let feel = super::feel::install_front(&mut world, &components)?;
    super::video::install(&mut world, engine_id, &layouts)?;
    crate::weather::install(&mut world, &layouts)?;
    // og.path and og.timeline after tweens, before og-anim (an animation
    // track sets players) and physics.
    super::timeline::install(&mut world, engine_id, &layouts, &ui)?;
    let contract = project.is_some_and(|p| p.look.is_some());
    let anim = super::anim::install(&mut world, super::art::step_default(contract, &layouts))?;
    let feel_records = super::feel::Records::of(&world, &layouts)?;
    super::feel::install_middle(
        &mut world,
        engine_id,
        &feel,
        &feel_records,
        &physics,
        profile,
    )?;
    super::e6::mount_back(&mut world, engine_id, profile, &physics)?;
    crate::destruction::set_physics(engine_id, &physics);
    let mounted = super::modules::mount_back(
        &mut world,
        engine_id,
        physics,
        ui,
        previous.map(|p| p.1),
        actions,
    )?;
    super::feel::install_back(&mut world, &feel, &feel_records)?;
    for name in lazy_named {
        crate::lazy::enable(&mut world, engine_id, name)?;
    }
    let names_by_id = world.schemas().into_iter().map(|s| s.name).collect();
    Ok(Assembled {
        world,
        probe: Some(probe),
        layouts,
        names_by_id,
        deferred,
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

/// Every record name game `g` declares it uses: its resources and what its
/// systems and hooks read, write and filter on.
fn named_by(g: &GameDecl, systems: &[SystemDecl], hooks: &[HookDecl]) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = g.resources.iter().cloned().collect();
    for s in g.systems.iter().filter_map(|&i| systems.get(i as usize)) {
        names.extend(s.reads.iter().chain(&s.writes).map(|t| t.name.clone()));
        names.extend(
            s.without
                .iter()
                .chain(&s.changed)
                .chain(&s.added)
                .chain(&s.removed)
                .cloned(),
        );
    }
    for h in g.hooks.iter().filter_map(|&i| hooks.get(i as usize)) {
        names.insert(h.component.clone());
        names.extend(h.reads.iter().chain(&h.writes).map(|t| t.name.clone()));
    }
    names
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
            #[cfg(not(feature = "profile2d"))]
            terrain: get("og.Terrain"),
            #[cfg(not(feature = "profile2d"))]
            scatter: get("og.TerrainScatter"),
            #[cfg(not(feature = "profile2d"))]
            grass: get("og.Grass"),
            #[cfg(not(feature = "profile2d"))]
            stamp: get("og.TerrainStamp"),
            #[cfg(not(feature = "profile2d"))]
            grass_pusher: get("og.GrassPusher"),
            #[cfg(not(feature = "profile2d"))]
            grass_type: get("og.GrassType"),
            collider: get("og.Collider"),
            lines: get("og.Lines"),
            points: get("og.Points"),
            probe: get("og.ReflectionProbe"),
            light_volume: get("og.LightVolume"),
            realtime_volume: get(crate::probes::REALTIME_LIGHT_VOLUME),
            interpolation: get("og.Interpolation"),
            origin: get(crate::origin::RESOURCE),
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
        deferred: a.deferred,
        resources: a.resources,
        systems: a.systems,
        commands: a.commands,
        names: BTreeMap::new(),
        name_of: IdMap::new(),
        name_ops: Vec::new(),
        rate: a.game.tick,
        max_catch_up: a.game.max_catch_up.max(1),
        accumulator_ms: 0.0,
        facts: a.game.facts,
        boot_scene: None,
        scene_source: None,
        scene_instances: Vec::new(),
        game_spawned: Default::default(),
        start: a.game.start,
        start_failed: None,
        project,
        boot_look: (!a.game.look.is_empty()).then(|| a.game.look.clone()),
        save_version: a.game.save_version,
        ui_desc: Value::Null,
        step_ms: 0.0,
        render_ms: 0.0,
        pass_ms: Vec::new(),
        gpu_ms: Vec::new(),
        triangles: 0,
        transparency: Default::default(),
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
        actions: Some(a.mounted.actions.clone()),
        physics_meshes: IdMap::new(),
        #[cfg(feature = "physics3d")]
        physics_baked: IdMap::new(),
        physics_heightfields: IdMap::new(),
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
    render.borrow_mut().engine = engine_id;
    let engine = Rc::new(Engine {
        id: engine_id,
        world: UnsafeCell::new(a.world),
        stepping: Cell::new(false),
        state: RefCell::new(state),
        assertions: probe.assertions(),
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
        super::timeline::commit(engine_id, probe.inbox());
        register_module_actions(&mut probe, engine_id);
    }
    // The host's locale (og run --locale, the browser's language).
    super::i18n::request(engine_id, options["locale"].as_str().unwrap_or(""));
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
    super::e6::retire(id);
    super::timeline::retire(id);
    crate::invariants::forget(id);
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
/// antialias, shadowMapSize, pointShadows, taaHistory, taaSharpen,
/// planarReflections, transparency and transparencyLayers. Both
/// hosts create their renderer from it, so the options mean the same on
/// each.
pub fn render_desc(id: u32, base: og_render::RenderDesc) -> og_render::RenderDesc {
    let options = render_options(id);
    let mut desc = base;
    desc.hdr = options["hdr"].as_bool().unwrap_or(false);
    desc.paper_white_nits = options["paperWhiteNits"].as_f64().unwrap_or(0.0) as f32;
    desc.peak_nits = options["peakNits"].as_f64().unwrap_or(0.0) as f32;
    desc.antialias = match options["antialias"].as_str().unwrap_or("tier") {
        "off" => og_render::Antialias::Off,
        "msaa4" => og_render::Antialias::Msaa4,
        "taa" => og_render::Antialias::Taa,
        "fxaa" => og_render::Antialias::Fxaa,
        "smaa" => og_render::Antialias::Smaa,
        "smaaT2x" => og_render::Antialias::SmaaT2x,
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
    desc.planar_reflections = options["planarReflections"].as_u64().map_or(1, |n| {
        n.clamp(1, u64::from(og_render::MAX_PLANAR_REFLECTIONS)) as u32
    });
    desc.transparency = match options["transparency"].as_str() {
        Some("linkedList") => og_render::Transparency::LinkedList,
        _ => og_render::Transparency::Weighted,
    };
    desc.transparency_layers = options["transparencyLayers"].as_u64().map_or(0, |n| {
        n.min(u64::from(og_render::MAX_TRANSPARENCY_LAYERS)) as u32
    });
    // og.motion times its compute slices where the adapter can (its
    // `Gpu::set_frame_budget`): a publish build asks only for a game whose
    // bundle declares the model.
    desc.timestamps = engine(id).is_ok_and(|e| crate::motion::present::declares(&e));
    desc
}

/// og-render's Antialias by its IDL name (RenderCapabilities.antialias).
pub fn antialias_name(a: og_render::Antialias) -> &'static str {
    match a {
        og_render::Antialias::Msaa4 => "msaa4",
        og_render::Antialias::Taa => "taa",
        og_render::Antialias::Fxaa => "fxaa",
        og_render::Antialias::Smaa => "smaa",
        og_render::Antialias::SmaaT2x => "smaaT2x",
        og_render::Antialias::Off | og_render::Antialias::Tier => "off",
    }
}

/// og-render's Transparency by its IDL name (RenderCapabilities.transparency).
pub fn transparency_name(t: og_render::Transparency) -> &'static str {
    match t {
        og_render::Transparency::Weighted => "weighted",
        og_render::Transparency::LinkedList => "linkedList",
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
