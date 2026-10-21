//! The engine modules mounted on every world beyond the core, as `hello`
//! lists them in `modules`: physics (og-physics), audio (og-audio) and the
//! UI (og-ui), with the binding symbols of the IDL's `physics` and `ui`
//! modules. Audio has no symbols: game code drives it through its
//! components, and the hosts' sinks read the command stream.
//!
//! The schedule a world gets, in order:
//!
//! 1. `og.physics.queries`: records the colliders, so `og.physics.raycast`
//!    and friends answer from inside the game's systems;
//! 2. `og.ui.begin`: opens the UI frame from `og.Input` and the viewport;
//! 3. the game's systems (declaration order, then `after`/`before`);
//! 4. `og.ui.end`: lays the frame out (the probe and the renderer read it);
//! 5. `og.physics`: steps the simulation and writes the dynamic bodies back;
//! 6. `og.ui.sound`: starts a voice for each UI event of the tick;
//! 7. `og.audio`: turns the tick's AudioSource changes into commands.
//!
//! The UI's interaction state is the `og.ui` state block, so snapshots and
//! the world hash carry it with the physics world (`og.physics`) and the
//! audio stream (its resources).

use super::engine::{Engine, engine, running};
use super::{ok, ok_void, parse, refuse};
use og_core::glam::Vec3;
use og_ecs::{Entity, StateBlock, SystemDesc, World};
use og_physics::{
    MeshDesc, OverlapDesc, Physics, PhysicsQuery, PhysicsSystem, QueryFilter, RaycastDesc,
    ShapeCastDesc, ShapeDesc,
};
use og_ui::{
    ActionButtonDesc, Align, Anchor, AnchorDesc, ButtonDesc, ButtonVariant, ChipDesc, ControlSize,
    CooldownDesc, Cross, DensityMode, DialogueDesc, Direction, DpadDesc, Element, FieldDesc,
    FontRole, Fonts, GridDesc, HueName, IconAtlas, IconDesc, ImageDesc, InputFrame, Insets,
    JoystickDesc, LabelDesc, ListDesc, ListRange, MapMarker, MeterDesc, MeterKind, MinimapDesc,
    ModalDesc, Nav, PanelDesc, Presence, ProgressDesc, RadioDesc, Response, Scheme, ScrollDesc,
    SelectDesc, SharedUi, Size, SkeletonDesc, SkinPart, SliderDesc, SpotlightDesc, Srgba,
    StepperDesc, TabsDesc, TextAlign, Theme, ToastDesc, Tone, TransitionDesc, TransitionKind,
    TypeStep, Ui, UiDesc, UiFont, Viewport,
};
use serde_json::{Map, Value, json};
use std::any::Any;
use std::cell::Cell;

/// The system that opens the UI frame, before the game's systems.
pub const UI_BEGIN: &str = "og.ui.begin";
/// The system that closes and lays out the UI frame, after them.
pub const UI_END: &str = "og.ui.end";
/// The state block that carries the UI's interaction state.
pub const UI_STATE: &str = "og.ui";
/// The system that plays the UI's event sounds (before og.audio, so they
/// start in the tick of their input).
pub const UI_SOUND: &str = "og.ui.sound";

thread_local! {
    /// The engine whose UI frame is open (between `og.ui.begin` and `og.ui.end`).
    static BUILDING: Cell<Option<u32>> = const { Cell::new(None) };
    /// The engine whose UI frame was opened last: where a call outside a
    /// frame logs its warning.
    static LAST: Cell<Option<u32>> = const { Cell::new(None) };
}

fn err(e: og_core::Error) -> String {
    e.reason().to_string()
}

/// What the modules hand the engine for one world.
pub(crate) struct Mounted {
    pub physics: PhysicsSystem,
    pub ui: SharedUi,
    pub audio: og_audio::AudioLink,
}

/// A UI with the theme and fonts of `previous` (a hot reload keeps them).
pub(crate) fn new_ui(tick: u32, previous: Option<&SharedUi>) -> SharedUi {
    let (theme, fonts, icons, skin) = match previous {
        Some(p) => {
            let ui = p.lock();
            (
                *ui.theme(),
                ui.fonts().clone(),
                ui.icons().clone(),
                ui.skin_name().to_string(),
            )
        }
        None => (
            Theme::default(),
            Fonts::default(),
            IconAtlas::default(),
            String::new(),
        ),
    };
    let mut ui = Ui::new(UiDesc {
        theme,
        ticks_per_second: tick,
        fonts,
    });
    ui.set_icons(icons);
    ui.set_skin_name(&skin);
    SharedUi::new(ui)
}

/// The modules' systems that run before the game's: physics' collider
/// recording and the UI's begin. Registers physics' types and settings.
pub(crate) fn mount_front(
    world: &mut World,
    engine_id: u32,
    profile: u32,
    ui: &SharedUi,
) -> Result<PhysicsSystem, String> {
    let physics = PhysicsSystem::register(
        world,
        Physics {
            profile,
            ..Physics::default()
        },
    )
    .map_err(err)?;
    let (desc, run) = physics.query_system();
    world.add_system(desc, run).map_err(err)?;
    let shared = ui.clone();
    world
        .add_system(
            SystemDesc::new(UI_BEGIN)
                .read::<og_probe::Input>()
                .read::<og_input::Gamepad>()
                .read::<og_input::Pointers>()
                .read::<og_input::TextInput>()
                .read::<og_input::A11yAction>(),
            move |ctx| {
                let pad = ctx.resource::<og_input::Gamepad>().ok().map(|g| *g);
                let pointers = ctx.resource::<og_input::Pointers>().ok().map(|p| *p);
                let typed = ctx.events::<og_input::TextInput>().unwrap_or_default();
                let asked = ctx.events::<og_input::A11yAction>().unwrap_or_default();
                let input = ctx
                    .resource::<og_probe::Input>()
                    .map(|i| InputFrame::read(&*i))
                    .unwrap_or_default()
                    .with(pad.as_ref(), pointers.as_ref())
                    .with_text(&typed)
                    .with_a11y(&asked);
                let viewport = viewport_of(engine_id);
                shared.lock().begin(input, viewport);
                BUILDING.with(|b| b.set(Some(engine_id)));
                LAST.with(|l| l.set(Some(engine_id)));
                Ok(())
            },
        )
        .map_err(err)?;
    Ok(physics)
}

/// The modules' systems that run after the game's: the UI's end, the
/// physics step and audio; then the UI's state block.
pub(crate) fn mount_back(
    world: &mut World,
    engine_id: u32,
    physics: PhysicsSystem,
    ui: SharedUi,
    link: Option<og_audio::AudioLink>,
) -> Result<Mounted, String> {
    let shared = ui.clone();
    world
        .add_system(SystemDesc::new(UI_END), move |_ctx| {
            BUILDING.with(|b| b.set(None));
            let warnings = shared.lock().end().warnings.clone();
            if !warnings.is_empty() {
                note_warnings(engine_id, warnings);
            }
            Ok(())
        })
        .map_err(err)?;
    super::physics_ext::schedule_ragdoll_targets(world, engine_id, &physics)?;
    physics.schedule(world).map_err(err)?;
    // UI sounds: the events this tick's UI queued (its begin, from the
    // input, and its widgets), played on the ui bus. A voice per event
    // (its entity kept in the UI state) restarts on a repeat. This runs
    // before og.audio, so og.audio turns the new sources into this tick's
    // commands: the sound starts in the tick of the input that caused it.
    let shared = ui.clone();
    world
        .add_system(
            SystemDesc::new(UI_SOUND).write::<og_audio::AudioSource>(),
            move |ctx| {
                let events = shared.lock().take_sounds();
                if events.is_empty() {
                    return Ok(());
                }
                for (event, sound) in events {
                    let slot = event as usize;
                    let old = shared.lock().voices_mut()[slot];
                    if old != 0 {
                        let e = Entity::from_bits(old);
                        if ctx.is_alive(e) {
                            ctx.despawn(e)?;
                        }
                    }
                    let e = ctx.spawn((og_audio::AudioSource {
                        bus: og_audio::Bus::Ui as u32,
                        ..og_audio::AudioSource::new(sound)
                    },))?;
                    shared.lock().voices_mut()[slot] = e.to_bits();
                }
                Ok(())
            },
        )
        .map_err(err)?;
    let audio = og_audio::install(
        world,
        &og_audio::AudioDesc {
            link,
            occluder: Some(super::e6::occluder(&physics)),
            ..og_audio::AudioDesc::default()
        },
    )
    .map_err(err)?;
    world
        .add_state_block(UI_STATE, Box::new(UiState(ui.clone())))
        .map_err(err)?;
    Ok(Mounted { physics, ui, audio })
}

/// A tick that failed between begin and end leaves no frame open.
pub(crate) fn end_of_ticks() {
    BUILDING.with(|b| b.set(None));
}

fn note_warnings(engine_id: u32, warnings: Vec<String>) {
    let Ok(eng) = engine(engine_id) else { return };
    let Ok(mut st) = eng.state.try_borrow_mut() else {
        return;
    };
    for w in warnings {
        if st.ui_warned.insert(w.clone()) {
            let tick = 0;
            st.log
                .push(json!({ "tick": tick, "warning": true, "text": format!("og.ui: {w}") }));
        }
    }
}

/// The UI's interaction state as world state.
struct UiState(SharedUi);

impl StateBlock for UiState {
    fn save(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.0.lock().save_state());
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ui::default().restore_state(bytes)?;
        Ok(Box::new(bytes.to_vec()))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let bytes = match decoded.and_then(|d| d.downcast::<Vec<u8>>().ok()) {
            Some(b) => *b,
            None => Ui::default().save_state(),
        };
        let _ = self.0.lock().restore_state(&bytes);
    }
}

// ── the viewport ────────────────────────────────────────────────────────────

/// The play surface an engine's UI lays out in: what the host set with
/// [`set_viewport`], else the render target at 1 CSS pixel per device
/// pixel, else 1280×720.
pub(crate) fn viewport_of(engine_id: u32) -> Viewport {
    if let Some(v) = preview_of(engine_id) {
        return v;
    }
    let Ok(eng) = engine(engine_id) else {
        return Viewport::default();
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return Viewport::default();
    };
    if let Some(v) = rs.viewport {
        return v;
    }
    match rs.backend.as_ref().map(|b| b.size()) {
        Some((w, h)) if w > 0 && h > 0 => Viewport::new(w as f32, h as f32),
        _ => Viewport::default(),
    }
}

/// The host's play surface: CSS pixels, device pixels per CSS pixel, the
/// safe area, the pointer kind, the page's motion, contrast and
/// transparency preferences and whether text controls take macOS's keys. The web host calls it when the canvas
/// is sized; native hosts take the render target's size.
pub fn set_viewport(engine_id: u32, viewport: Value) -> Result<(), String> {
    let eng = engine(engine_id)?;
    let num = |k: &str, d: f64| viewport[k].as_f64().unwrap_or(d) as f32;
    let v = Viewport {
        width: num("width", 1280.0).max(1.0),
        height: num("height", 720.0).max(1.0),
        dpr: num("dpr", 1.0),
        safe_area: insets(&viewport["safeArea"]),
        coarse_pointer: viewport["coarsePointer"].as_bool().unwrap_or(false),
        reduced_motion: viewport["reducedMotion"].as_bool().unwrap_or(false),
        high_contrast: viewport["highContrast"].as_bool().unwrap_or(false),
        reduced_transparency: viewport["reducedTransparency"].as_bool().unwrap_or(false),
        mac_keys: viewport["macKeys"].as_bool().unwrap_or(false),
    };
    eng.render.borrow_mut().viewport = Some(v);
    Ok(())
}

fn viewport_json(v: &Viewport) -> Value {
    json!({
        "width": v.width, "height": v.height, "dpr": v.dpr,
        "safeArea": { "left": v.safe_area.left, "top": v.safe_area.top, "right": v.safe_area.right, "bottom": v.safe_area.bottom },
        "coarsePointer": v.coarse_pointer,
        "reducedMotion": v.reduced_motion,
        "highContrast": v.high_contrast,
        "reducedTransparency": v.reduced_transparency,
        "macKeys": v.mac_keys,
        "breakpoint": v.breakpoint().name(),
    })
}

// ── physics ─────────────────────────────────────────────────────────────────

fn vec3(v: &Value, d: Vec3) -> Vec3 {
    if !v.is_object() {
        return d;
    }
    let n = |k: &str, x: f32| v[k].as_f64().map_or(x, |f| f as f32);
    Vec3::new(n("x", d.x), n("y", d.y), n("z", d.z))
}

fn quat(v: &Value) -> og_core::glam::Quat {
    if !v.is_object() {
        return og_core::glam::Quat::IDENTITY;
    }
    let n = |k: &str, x: f32| v[k].as_f64().map_or(x, |f| f as f32);
    og_core::glam::Quat::from_xyzw(n("x", 0.0), n("y", 0.0), n("z", 0.0), n("w", 1.0))
}

fn v3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn filter(v: &Value) -> Result<QueryFilter, String> {
    let exclude = match v["exclude"].as_f64() {
        Some(n) if n >= 1.0 && n.fract() == 0.0 => Some(Entity::from_bits(n as u64)),
        Some(n) => return Err(format!("filter.exclude: {n} is not an entity")),
        None => None,
    };
    Ok(QueryFilter {
        mask: v["mask"].as_u64().unwrap_or(0) as u32,
        include_sensors: v["includeSensors"].as_bool().unwrap_or(false),
        exclude,
    })
}

fn shape(v: &Value) -> ShapeDesc {
    let d = ShapeDesc::default();
    let n = |k: &str, x: f32| v[k].as_f64().map_or(x, |f| f as f32);
    ShapeDesc {
        shape: v["shape"].as_u64().map_or(d.shape, |s| s as u32),
        size: vec3(&v["size"], d.size),
        radius: n("radius", d.radius),
        height: n("height", d.height),
        mesh: v["mesh"].as_f64().map_or(d.mesh, |m| m as u64),
    }
}

/// The query view a call sees: the colliders recorded at the start of the
/// tick inside a system, the world as it is between ticks.
fn query(world: u32) -> Result<PhysicsQuery, String> {
    let eng = engine(world)?;
    let physics = eng
        .state
        .borrow()
        .physics
        .clone()
        .ok_or("this engine has no physics")?;
    if running(world).is_some() {
        return physics.tick_query().map_err(err);
    }
    physics.query(eng.world()?).map_err(err)
}

pub fn og_physics_raycast(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let d = parse(desc.as_deref())?;
        let desc = RaycastDesc {
            origin: vec3(&d["origin"], Vec3::ZERO),
            direction: vec3(&d["direction"], Vec3::NEG_Y),
            max_distance: d["maxDistance"].as_f64().unwrap_or(0.0) as f32,
            filter: filter(&d["filter"])?,
        };
        let hit = query(world)?.raycast(&desc).map_err(err)?;
        Ok(hit.map_or(Value::Null, |h| {
            json!({ "entity": h.entity.to_bits(), "distance": h.distance, "point": v3(h.point), "normal": v3(h.normal) })
        }))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_physics_shape_cast(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let d = parse(desc.as_deref())?;
        let desc = ShapeCastDesc {
            shape: shape(&d["shape"]),
            position: vec3(&d["position"], Vec3::ZERO),
            rotation: quat(&d["rotation"]),
            direction: vec3(&d["direction"], Vec3::NEG_Y),
            max_distance: d["maxDistance"].as_f64().unwrap_or(0.0) as f32,
            filter: filter(&d["filter"])?,
        };
        let hit = query(world)?.shape_cast(&desc).map_err(err)?;
        Ok(hit.map_or(Value::Null, |h| {
            json!({ "entity": h.entity.to_bits(), "distance": h.distance, "point": v3(h.point), "normal": v3(h.normal) })
        }))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_physics_overlap(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let d = parse(desc.as_deref())?;
        let desc = OverlapDesc {
            shape: shape(&d["shape"]),
            position: vec3(&d["position"], Vec3::ZERO),
            rotation: quat(&d["rotation"]),
            filter: filter(&d["filter"])?,
        };
        let hits = query(world)?.overlap(&desc).map_err(err)?;
        Ok(json!(hits.iter().map(|e| e.to_bits()).collect::<Vec<_>>()))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// Registers a mesh with the engine's physics and remembers it, so a hot
/// reload's new physics gets it too.
pub(crate) fn set_mesh(eng: &Engine, handle: u64, mesh: MeshDesc) -> Result<(), String> {
    let physics = eng
        .state
        .borrow()
        .physics
        .clone()
        .ok_or("this engine has no physics")?;
    physics.set_mesh(handle, mesh.clone()).map_err(err)?;
    eng.state.borrow_mut().physics_meshes.insert(handle, mesh);
    Ok(())
}

pub fn og_physics_set_mesh(world: u32, mesh: f64, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        if mesh.fract() != 0.0 || !(0.0..9_007_199_254_740_992.0).contains(&mesh) {
            return Err(format!("{mesh} is not a geometry handle"));
        }
        let positions: Vec<Vec3> = d["positions"]
            .as_array()
            .map(|a| a.iter().map(|p| vec3(p, Vec3::ZERO)).collect())
            .unwrap_or_default();
        let flat: Vec<u32> = d["indices"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|i| i.as_u64().map(|n| n as u32))
                    .collect()
            })
            .unwrap_or_default();
        if !flat.len().is_multiple_of(3) {
            return Err(format!(
                "MeshDesc.indices holds {} numbers: triangles are triples",
                flat.len()
            ));
        }
        let indices = flat.chunks(3).map(|t| [t[0], t[1], t[2]]).collect();
        set_mesh(&eng, mesh as u64, MeshDesc { positions, indices })
    })();
    match answer {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// The geometry of a render mesh, as physics takes it.
pub(crate) fn mesh_of(data: &og_render::MeshData) -> MeshDesc {
    MeshDesc {
        positions: data
            .vertices
            .iter()
            .map(|v| Vec3::from(v.position))
            .collect(),
        indices: data
            .indices
            .chunks(3)
            .filter(|t| t.len() == 3)
            .map(|t| [t[0], t[1], t[2]])
            .collect(),
    }
}

// ── ui: decoding descs ──────────────────────────────────────────────────────

fn f(v: &Value, k: &str, d: f32) -> f32 {
    v[k].as_f64().map_or(d, |x| x as f32)
}

fn insets(v: &Value) -> Insets {
    Insets {
        left: f(v, "left", 0.0),
        top: f(v, "top", 0.0),
        right: f(v, "right", 0.0),
        bottom: f(v, "bottom", 0.0),
    }
}

fn size(v: &Value) -> Option<Size> {
    let range = |v: &Value| (f(v, "min", 0.0), f(v, "max", f32::INFINITY));
    Some(match v["kind"].as_str()? {
        "fit" => {
            let (min, max) = range(v);
            Size::Fit { min, max }
        }
        "grow" => {
            let (min, max) = range(v);
            Size::Grow { min, max }
        }
        "fixed" => Size::Fixed(f(v, "px", 0.0)),
        "percent" => Size::Percent(f(v, "fraction", 0.0)),
        _ => return None,
    })
}

fn size_or(v: &Value, d: Size) -> Size {
    size(v).unwrap_or(d)
}

/// A palette colour by its ThemeColor name.
fn token(theme: &Theme, name: &str) -> Option<Srgba> {
    let p = &theme.palette;
    Some(match name {
        "accent" => p.accent,
        "accentActive" => p.accent_active,
        "accentFg" => p.accent_fg,
        "accentHover" => p.accent_hover,
        "accentInk" => p.accent_ink,
        "accentSoft" => p.accent_soft,
        "bg" => p.bg,
        "bgSubtle" => p.bg_subtle,
        "border" => p.border,
        "borderStrong" => p.border_strong,
        "danger" => p.danger,
        "fg" => p.fg,
        "fgInverse" => p.fg_inverse,
        "fgMuted" => p.fg_muted,
        "fgSubtle" => p.fg_subtle,
        "fill" => p.fill,
        "fillStrong" => p.fill_strong,
        "focus" => p.focus,
        "info" => p.info,
        "materialControl" => p.material_control,
        "materialControlHover" => p.material_control_hover,
        "materialEdge" => p.material_edge,
        "materialGlow" => p.material_glow,
        "materialRegular" => p.material_regular,
        "materialRimSoft" => p.material_rim_soft,
        "materialSheen" => p.material_sheen,
        "materialShine" => p.material_shine,
        "materialThick" => p.material_thick,
        "materialThin" => p.material_thin,
        "scrim" => p.scrim,
        "shadow" => p.shadow,
        "sheen" => p.sheen,
        "stage" => p.stage,
        "stageBorder" => p.stage_border,
        "stageDanger" => p.stage_danger,
        "stageFg" => p.stage_fg,
        "stageMuted" => p.stage_muted,
        "stageRaised" => p.stage_raised,
        "stageSurface" => p.stage_surface,
        "success" => p.success,
        "surface" => p.surface,
        "surfaceHover" => p.surface_hover,
        "surfaceRaised" => p.surface_raised,
        "thumb" => p.thumb,
        "warn" => p.warn,
        _ => return None,
    })
}

/// Every ThemeColor name, in the IDL's order.
pub const THEME_COLORS: &[&str] = &[
    "accent",
    "accentActive",
    "accentFg",
    "accentHover",
    "accentInk",
    "accentSoft",
    "bg",
    "bgSubtle",
    "border",
    "borderStrong",
    "danger",
    "fg",
    "fgInverse",
    "fgMuted",
    "fgSubtle",
    "fill",
    "fillStrong",
    "focus",
    "info",
    "materialControl",
    "materialControlHover",
    "materialEdge",
    "materialGlow",
    "materialRegular",
    "materialRimSoft",
    "materialSheen",
    "materialShine",
    "materialThick",
    "materialThin",
    "scrim",
    "shadow",
    "sheen",
    "stage",
    "stageBorder",
    "stageDanger",
    "stageFg",
    "stageMuted",
    "stageRaised",
    "stageSurface",
    "success",
    "surface",
    "surfaceHover",
    "surfaceRaised",
    "thumb",
    "warn",
];

/// A type step by its TypeStepName.
fn step(theme: &Theme, name: &str) -> Option<TypeStep> {
    let r = &theme.type_ramp;
    Some(match name {
        "body" => r.body,
        "xs" => r.xs,
        "meta" => r.meta,
        "caption" => r.caption,
        "sm" => r.sm,
        "reading" => r.reading,
        "base" => r.base,
        "lg" => r.lg,
        "xl" => r.xl,
        "xl2" => r.xl2,
        "xl3" => r.xl3,
        "xl4" => r.xl4,
        "xl5" => r.xl5,
        "xl6" => r.xl6,
        "xl7" => r.xl7,
        "xl8" => r.xl8,
        "displaySm" => r.display_sm,
        "displayMd" => r.display_md,
        "displayLg" => r.display_lg,
        "displayXl" => r.display_xl,
        "displayXl2" => r.display_xl2,
        _ => return None,
    })
}

/// Every TypeStepName, in the IDL's order.
pub const TYPE_STEPS: &[&str] = &[
    "body",
    "xs",
    "meta",
    "caption",
    "sm",
    "reading",
    "base",
    "lg",
    "xl",
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "displaySm",
    "displayMd",
    "displayLg",
    "displayXl",
    "displayXl2",
];

/// A Paint: a token, else an sRGB colour as authored (0–1), else white.
fn paint(theme: &Theme, v: &Value) -> Option<Srgba> {
    if !v.is_object() {
        return None;
    }
    let alpha = f(v, "alpha", 1.0).clamp(0.0, 1.0);
    let base = match v["token"].as_str() {
        Some(t) => token(theme, t)?,
        None => {
            let c = &v["color"];
            if c.is_object() {
                let ch = |k: &str| (f(c, k, 1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
                Srgba::rgb(ch("r"), ch("g"), ch("b"))
            } else {
                Srgba::WHITE
            }
        }
    };
    Some(base.fade(alpha))
}

fn element(theme: &Theme, d: &Value) -> Element {
    Element {
        direction: match d["direction"].as_str() {
            Some("column") => Direction::Column,
            _ => Direction::Row,
        },
        width: size_or(&d["width"], Size::fit()),
        height: size_or(&d["height"], Size::fit()),
        padding: insets(&d["padding"]),
        gap: f(d, "gap", 0.0),
        align: align(&d["align"]),
        cross: cross(&d["cross"]),
        wrap: d["wrap"].as_bool().unwrap_or(false),
        scroll: d["scroll"].as_bool().unwrap_or(false),
        scroll_x: d["scrollX"].as_bool().unwrap_or(false),
        interactive: d["interactive"].as_bool().unwrap_or(false),
        background: paint(theme, &d["background"]),
        radius: f(d, "radius", 0.0),
        skin: skin_part(&d["frame"]),
        aspect: f(d, "aspect", 0.0).max(0.0),
    }
}

fn skin_part(v: &Value) -> Option<SkinPart> {
    SkinPart::from_name(v.as_str()?)
}

fn align(v: &Value) -> Align {
    match v.as_str() {
        Some("center") => Align::Center,
        Some("end") => Align::End,
        Some("spaceBetween") => Align::SpaceBetween,
        _ => Align::Start,
    }
}

fn cross(v: &Value) -> Cross {
    match v.as_str() {
        Some("center") => Cross::Center,
        Some("end") => Cross::End,
        Some("stretch") => Cross::Stretch,
        _ => Cross::Start,
    }
}

fn response(r: Response) -> Value {
    json!({
        "hovered": r.hovered, "pressed": r.pressed, "clicked": r.clicked,
        "changed": r.changed, "focused": r.focused, "submitted": r.submitted,
        "dropped": r.dropped, "pad": r.pad, "focusedBy": r.focused_by,
    })
}

// ── ui: the calls ───────────────────────────────────────────────────────────

/// Runs `f` on the UI whose frame is open; outside a frame, logs a warning
/// on the engine that opened one last and answers `None`.
fn with_ui<T>(call: &str, f: impl FnOnce(&mut Ui) -> T) -> Option<T> {
    let building = BUILDING.with(Cell::get);
    let target = building.or_else(|| LAST.with(Cell::get));
    let eng = target.and_then(|id| engine(id).ok())?;
    if building.is_none() {
        if let Ok(mut st) = eng.state.try_borrow_mut() {
            let text = format!(
                "{call} was called outside a system: og.ui calls build the frame the engine opens before the game's systems and closes after them, so they do nothing anywhere else"
            );
            if st.ui_warned.insert(text.clone()) {
                st.log
                    .push(json!({ "tick": 0, "warning": true, "text": text }));
            }
        }
        return None;
    }
    let ui = eng.state.borrow().ui.clone()?;
    let mut guard = ui.lock();
    Some(f(&mut guard))
}

/// The UiDesc a configure call makes: the last one's fields with the ones
/// this call gives (`null` clears a field; one left out keeps its value).
fn merged_ui_desc(last: &Value, given: Value) -> Result<Value, String> {
    let mut d = match last {
        Value::Object(o) => o.clone(),
        _ => Map::new(),
    };
    match given {
        Value::Object(o) => d.extend(o),
        Value::Null => {}
        other => return Err(format!("desc: a UiDesc is an object, not {other}")),
    }
    Ok(Value::Object(d))
}

pub fn og_ui_configure(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let eng = engine(world)?;
        let d = merged_ui_desc(&eng.state.borrow().ui_desc, parse(desc.as_deref())?)?;
        let skin = match d["skin"].as_f64() {
            Some(h) => {
                let i =
                    super::assets::index_of(h).ok_or_else(|| format!("skin: {h} is not a skin"))?;
                let a = eng.assets.borrow();
                let s = a
                    .skins
                    .get(i)
                    .ok_or_else(|| format!("skin: skin {h} is not alive"))?;
                Some((s.file.clone(), s.atlas, s.fonts, s.icons, s.sounds))
            }
            None => None,
        };
        let scheme = match d["scheme"].as_str() {
            Some("light") => Scheme::Light,
            _ => Scheme::Dark,
        };
        let mut theme = match &skin {
            Some((file, atlas, _, _, sounds)) => file.theme(*atlas, *sounds),
            None => Theme::new(scheme),
        };
        if d["contrast"].as_str() == Some("high") {
            theme = theme.to_high_contrast();
        }
        if let Some(h) = d["accentHue"].as_str().and_then(HueName::from_name) {
            theme = theme.with_hue(h);
        } else if d["accentColor"].is_object() {
            let c = &d["accentColor"];
            let ch = |k: &str| (f(c, k, 1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
            theme = theme.with_accent(Srgba::rgb(ch("r"), ch("g"), ch("b")));
        }
        theme.reduced_motion = d["reducedMotion"].as_bool().unwrap_or(false);
        theme.reduced_transparency = d["reducedTransparency"].as_bool().unwrap_or(false);
        theme.density = match d["density"].as_str() {
            Some("compact") => DensityMode::Compact,
            Some("comfortable") => DensityMode::Comfortable,
            Some("touch") => DensityMode::Touch,
            _ => DensityMode::Auto,
        };
        let sounds = &d["sounds"];
        if sounds.is_object() {
            for e in og_ui::UiSound::ALL {
                if let Some(h) = sounds[e.name()].as_f64() {
                    theme.skin.sounds[e as usize] = h as u64;
                }
            }
        }
        let font = |key: &str| -> Result<Option<UiFont>, String> {
            match d[key].as_f64() {
                None => Ok(None),
                Some(h) => ui_font(&eng, h).map(Some),
            }
        };
        let skin_font = |i: usize| -> Result<Option<UiFont>, String> {
            match skin.as_ref().and_then(|s| s.2[i]) {
                None => Ok(None),
                Some(h) => ui_font(&eng, h).map(Some),
            }
        };
        let body = match font("font")? {
            Some(f) => Some(f),
            None => skin_font(0)?,
        };
        let fonts = Fonts {
            body: body.unwrap_or_default(),
            display: match font("displayFont")? {
                Some(f) => Some(f),
                None => skin_font(1)?,
            },
            mono: match font("monoFont")? {
                Some(f) => Some(f),
                None => skin_font(2)?,
            },
            rounded: font("roundedFont")?,
            fallbacks: match d["fallbackFonts"].as_array() {
                Some(list) => list
                    .iter()
                    .filter_map(Value::as_f64)
                    .map(|h| ui_font(&eng, h))
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
            },
            fallback: skin_font(3)?,
            // The locale's chain (og.i18n) stays.
            chain: Vec::new(),
        };
        let ui = eng
            .state
            .borrow()
            .ui
            .clone()
            .ok_or("this engine has no UI")?;
        let icons = &d["icons"];
        let atlas = if icons.is_object() {
            let texture = match icons["texture"].as_f64() {
                Some(t) => {
                    let rs = eng.render.borrow();
                    let i = super::assets::index_of(t)
                        .ok_or_else(|| format!("icons.texture: {t} is not a texture"))?;
                    Some(
                        *rs.textures
                            .get(i)
                            .ok_or_else(|| format!("icons.texture: texture {t} is not alive"))?,
                    )
                }
                None => None,
            };
            // An icon set (an `.ogt` of `type icons`, loaded as a font):
            // distance-field glyphs, named by the set.
            let set = match icons["font"].as_f64() {
                Some(h) => {
                    let i = super::assets::index_of(h)
                        .ok_or_else(|| format!("icons.font: {h} is not a font"))?;
                    let names = eng
                        .assets
                        .borrow()
                        .fonts
                        .get(i)
                        .map(|f| f.icons.clone())
                        .ok_or_else(|| format!("icons.font: font {h} is not alive"))?;
                    if names.is_empty() {
                        return Err(format!(
                            "icons.font: font {h} is not an icon set (cook SVGs with an .ogt of `type icons`)"
                        ));
                    }
                    Some((ui_font(&eng, h)?, names))
                }
                None => None,
            };
            match set {
                Some((font, names)) => IconAtlas {
                    font: Some(font),
                    names,
                    ..IconAtlas::default()
                },
                None => IconAtlas {
                    texture,
                    columns: icons["columns"].as_u64().unwrap_or(1).max(1) as u32,
                    rows: icons["rows"].as_u64().unwrap_or(1).max(1) as u32,
                    names: icons["names"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .map(|n| n.as_str().unwrap_or_default().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                    font: None,
                },
            }
        } else if let Some((file, _, _, Some(texture), _)) = &skin {
            let (_, columns, rows, names) = file.icons.clone().unwrap_or_default();
            IconAtlas {
                texture: Some(*texture),
                columns,
                rows,
                names,
                font: None,
            }
        } else {
            IconAtlas::default()
        };
        let mut ui = ui.lock();
        ui.set_theme(theme);
        let chain = ui.fonts().chain.clone();
        let fonts = Fonts { chain, ..fonts };
        let fontless = fonts.body.handle.is_none()
            && fonts.display.is_none()
            && fonts.mono.is_none()
            && fonts.rounded.is_none()
            && fonts.fallbacks.is_empty()
            && fonts.fallback.is_none()
            && fonts.chain.is_empty();
        ui.set_fonts(fonts);
        ui.set_icons(atlas);
        ui.set_skin_name(skin.as_ref().map_or("", |s| s.0.name.as_str()));
        ui.set_toast_anchor(anchor_of(&d["toastAnchor"]).unwrap_or(Anchor::Bottom));
        drop(ui);
        let mut st = eng.state.borrow_mut();
        st.ui_desc = d;
        if fontless {
            let text = "og.ui.configure: the UI has no font now, so it draws no text (every label, button and field is blank): give `font` (og.load.font) or a `skin` (og.load.skin, whose fonts it brings), in this call or an earlier one; a field set to null is cleared, a field left out keeps its value".to_string();
            if st.ui_warned.insert(text.clone()) {
                let tick = eng.world().map_or(0, |w| w.time().tick);
                st.log
                    .push(json!({ "tick": tick, "warning": true, "text": text }));
            }
        }
        Ok(())
    })();
    match answer {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// A loaded skin: its file and the handles of the assets it names.
pub(crate) struct LoadedSkin {
    pub id: String,
    pub file: og_ui::SkinFile,
    pub atlas: Option<og_render::TextureHandle>,
    /// Font asset handles: body, display, numeric, fallback.
    pub fonts: [Option<f64>; 4],
    pub icons: Option<og_render::TextureHandle>,
    pub sounds: [u64; 6],
}

/// `og.load.skin`: a cooked skin, its atlas, fonts, icons and sounds loaded with it.
pub fn og_load_skin(world: u32, path: String) -> String {
    match engine(world).and_then(|eng| load_skin(&eng, &path)) {
        Ok(i) => ok(json!(super::assets::handle(i) as f64)),
        Err(e) => refuse(e),
    }
}

/// `og.load.skin`: the skin's load index, its atlas, fonts, icons and
/// sounds loaded with it (once per path).
pub(crate) fn load_skin(eng: &Engine, path: &str) -> Result<usize, String> {
    (|| -> Result<usize, String> {
        let id = og_cook::runtime::asset_id(path).to_string();
        if let Some(i) = eng.assets.borrow().skins.iter().position(|s| s.id == id) {
            return Ok(i);
        }
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .skin_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let mut file = og_ui::parse_skin(&text).map_err(|e| format!("{id}: {e}"))?;
        // Ids relative to the skin file first, as the cook resolved them.
        file.resolve(&id, |r| eng.assets.borrow().bundle_for(r).is_ok());
        let texture = |p: &str| -> Result<og_render::TextureHandle, String> {
            let i = super::assets::load_texture(eng, p)?;
            eng.render
                .borrow()
                .textures
                .get(i)
                .copied()
                .ok_or_else(|| format!("{id}: the texture {p} did not load"))
        };
        let atlas = file.atlas.as_deref().map(texture).transpose()?;
        let icons = file.icons.as_ref().map(|i| texture(&i.0)).transpose()?;
        let mut fonts = [None; 4];
        for (k, p) in file.fonts.iter().enumerate() {
            if let Some(p) = p {
                let i = super::assets::load_font(eng, p)?;
                fonts[k] = Some(super::assets::handle(i) as f64);
            }
        }
        let mut sounds = [0u64; 6];
        for (k, p) in file.sounds.iter().enumerate() {
            if let Some(p) = p {
                let i = super::assets::load_sound(eng, p)?;
                sounds[k] = eng.assets.borrow().sound_handles[i];
            }
        }
        let mut a = eng.assets.borrow_mut();
        a.skins.push(LoadedSkin {
            id,
            file,
            atlas,
            fonts,
            icons,
            sounds,
        });
        Ok(a.skins.len() - 1)
    })()
}

/// A loaded font as the UI lays out and draws with it: its first atlas
/// page's handle and metrics (every page carries every glyph's advance);
/// the renderer draws the other pages from the same layout.
pub(crate) fn ui_font(eng: &Engine, handle: f64) -> Result<UiFont, String> {
    let index = super::assets::index_of(handle).ok_or_else(|| format!("{handle} is not a font"))?;
    let (render, file) = eng
        .assets
        .borrow()
        .fonts
        .get(index)
        .map(|f| (f.render, f.file.clone()))
        .ok_or_else(|| format!("font {handle} is not alive"))?;
    let mut rs = eng.render.borrow_mut();
    let pages = rs
        .fonts
        .get(render)
        .map(|p| p.pages.clone())
        .ok_or_else(|| format!("font {handle} has no atlas"))?;
    let first = *pages.first().ok_or("the font has no pages")?;
    let msdf = rs
        .scene
        .font(first)
        .cloned()
        .ok_or("the font's first page is missing")?;
    rs.ui_pages.insert(first, pages);
    let mut font = UiFont::new(first, &msdf);
    font.metrics = font.metrics.with_file(file);
    Ok(font)
}

pub fn og_ui_viewport(world: u32) -> String {
    viewport_json(&viewport_of(world)).to_string()
}

pub fn og_ui_theme(world: u32) -> String {
    let (theme, sizes, skin) = engine(world)
        .ok()
        .and_then(|e| e.state.borrow().ui.clone())
        .map(|ui| {
            let ui = ui.lock();
            (*ui.theme(), ui.sizes(), ui.skin_name().to_string())
        })
        .unwrap_or_else(|| {
            let t = Theme::default();
            let s = t.sizes(&Viewport::default());
            (t, s, String::new())
        });
    theme_json(theme, sizes, &skin)
}

pub fn og_ui_frame_viewport() -> String {
    viewport_json(&with_ui("og.ui.frameViewport", |ui| ui.viewport()).unwrap_or_default())
        .to_string()
}

pub fn og_ui_has_icon(name: String) -> bool {
    with_ui("og.ui.hasIcon", |ui| ui.has_icon(&name)).unwrap_or(false)
}

pub fn og_ui_seconds() -> f64 {
    with_ui("og.ui.seconds", |ui| ui.seconds()).unwrap_or(0.0)
}

pub fn og_ui_frame_theme() -> String {
    let (theme, sizes, skin) = with_ui("og.ui.frameTheme", |ui| {
        (*ui.theme(), ui.sizes(), ui.skin_name().to_string())
    })
    .unwrap_or_else(|| {
        let t = Theme::default();
        let s = t.sizes(&Viewport::default());
        (t, s, String::new())
    });
    theme_json(theme, sizes, &skin)
}

fn theme_json(theme: Theme, sizes: og_ui::Density, skin: &str) -> String {
    let hex = |c: Srgba| {
        let [r, g, b] = c.rgb;
        if c.alpha >= 1.0 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!(
                "#{r:02x}{g:02x}{b:02x}{:02x}",
                (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8
            )
        }
    };
    let mut palette = Map::new();
    for name in THEME_COLORS {
        if let Some(c) = token(&theme, name) {
            palette.insert(name.to_string(), json!(hex(c)));
        }
    }
    let mut ramp = Map::new();
    for name in TYPE_STEPS {
        if let Some(s) = step(&theme, name) {
            ramp.insert(
                name.to_string(),
                json!({ "size": s.size, "lineHeight": s.line_height, "trackingEm": s.tracking_em, "cjkLineHeight": s.cjk_line_height }),
            );
        }
    }
    let sp = theme.space;
    let r = theme.radius;
    let d = theme.durations_ms;
    json!({
        "scheme": if theme.scheme == Scheme::Light { "light" } else { "dark" },
        "palette": palette,
        "space": { "xxs": sp.xxs, "xs": sp.xs, "sm": sp.sm, "md": sp.md, "lg": sp.lg, "xl": sp.xl, "xxl": sp.xxl, "xl3": sp.xl3 },
        "radius": { "control": r.control, "field": r.field, "block": r.block, "card": r.card, "panel": r.panel, "sheet": r.sheet, "button": r.button },
        "typeRamp": ramp,
        "sizes": { "controlSm": sizes.control_sm, "controlMd": sizes.control_md, "controlLg": sizes.control_lg, "gutter": sizes.gutter, "header": sizes.header, "row": sizes.row, "section": sizes.section },
        "durationsMs": { "instant": d.instant, "fast": d.fast, "base": d.base, "slow": d.slow, "spring": d.spring, "bounce": d.bounce },
        "skin": skin,
        "contrast": if theme.contrast == og_ui::Contrast::High { "high" } else { "standard" },
        "reducedMotion": theme.reduced_motion,
        "reducedTransparency": theme.reduced_transparency,
    })
    .to_string()
}

fn desc_of(desc: Option<String>) -> Value {
    parse(desc.as_deref()).unwrap_or(Value::Null)
}

pub fn og_ui_open(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.open", |ui| {
        let el = element(ui.theme(), &d);
        ui.open(id.as_str(), el)
    });
}

pub fn og_ui_open_panel(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.openPanel", |ui| {
        let t = *ui.theme();
        let p = PanelDesc {
            row: d["row"].as_bool().unwrap_or(false),
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            padding: d["padding"].is_object().then(|| insets(&d["padding"])),
            gap: d["gap"].as_f64().map(|g| g as f32),
            align: align(&d["align"]),
            cross: cross(&d["cross"]),
            background: paint(&t, &d["background"]),
        };
        ui.open_panel(id.as_str(), p)
    });
}

pub fn og_ui_open_scroll(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.openScroll", |ui| {
        let s = ScrollDesc {
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            padding: insets(&d["padding"]),
            gap: f(&d, "gap", 0.0),
            axes: match d["axes"].as_str() {
                Some("horizontal") => og_ui::ScrollAxes::Horizontal,
                Some("both") => og_ui::ScrollAxes::Both,
                _ => og_ui::ScrollAxes::Vertical,
            },
        };
        ui.open_scroll(id.as_str(), s)
    });
}

pub fn og_ui_close() -> String {
    response(with_ui("og.ui.close", |ui| ui.close()).unwrap_or_default()).to_string()
}

pub fn og_ui_label(id: String, text: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.label", |ui| {
        let t = *ui.theme();
        let l = LabelDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            color: paint(&t, &d["color"]),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            nowrap: d["nowrap"].as_bool().unwrap_or(false),
            width: size_or(&d["width"], Size::fit()),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
            heading: d["heading"].as_bool().unwrap_or(false),
            ellipsis: d["ellipsis"].as_bool().unwrap_or(false),
            max_lines: d["maxLines"].as_u64().unwrap_or(0) as u32,
        };
        ui.label(id.as_str(), &text, l)
    });
}

pub fn og_ui_rich_text(id: String, markup: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.richText", |ui| {
        let t = *ui.theme();
        let desc = og_ui::RichTextDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            color: paint(&t, &d["color"]),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            nowrap: d["nowrap"].as_bool().unwrap_or(false),
            width: size_or(&d["width"], Size::fit()),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
        };
        ui.rich_text(id.as_str(), &markup, desc)
    })
    .unwrap_or_default();
    json!({ "response": response(r.response), "link": r.link, "hovered": r.hovered }).to_string()
}

pub fn og_ui_button(id: String, text: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.button", |ui| {
        let b = ButtonDesc {
            variant: match d["variant"].as_str() {
                Some("primary") => ButtonVariant::Primary,
                Some("quiet") => ButtonVariant::Quiet,
                Some("danger") => ButtonVariant::Danger,
                _ => ButtonVariant::Secondary,
            },
            size: match d["size"].as_str() {
                Some("small") => ControlSize::Small,
                Some("large") => ControlSize::Large,
                _ => ControlSize::Medium,
            },
            disabled: d["disabled"].as_bool().unwrap_or(false),
            width: size(&d["width"]),
            autofocus: d["autofocus"].as_bool().unwrap_or(false),
        };
        ui.button(id.as_str(), &text, b)
    });
    response(r.unwrap_or_default()).to_string()
}

pub fn og_ui_image(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    let texture = d["texture"].as_f64().and_then(|t| {
        let eng = engine(BUILDING.with(Cell::get)?).ok()?;
        let rs = eng.render.borrow();
        super::assets::index_of(t).and_then(|i| rs.textures.get(i).copied())
    });
    with_ui("og.ui.image", |ui| {
        let t = *ui.theme();
        let uv = d["uv"].as_array().filter(|a| a.len() == 4).map(|a| {
            let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
            [n(0), n(1), n(2), n(3)]
        });
        let i = ImageDesc {
            // A texture-less image is a solid rectangle.
            texture,
            uv,
            tint: paint(&t, &d["tint"]),
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            aspect: f(&d, "aspect", 0.0).max(0.0),
        };
        ui.image(id.as_str(), i)
    });
}

pub fn og_ui_zoom_view(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let texture = texture_of(&d["texture"]);
    let r = with_ui("og.ui.zoomView", |ui| {
        let t = *ui.theme();
        ui.zoom_view(
            id.as_str(),
            &og_ui::ZoomViewDesc {
                texture,
                uv: uv_of(&d["uv"]),
                tint: paint(&t, &d["tint"]),
                width: size_or(&d["width"], Size::fit()),
                height: size_or(&d["height"], Size::fit()),
                aspect: f(&d, "aspect", 0.0).max(0.0),
                min_zoom: f(&d, "minZoom", 0.0),
                max_zoom: f(&d, "maxZoom", 0.0),
            },
        )
    });
    let (zoom, x, y) = r.map_or((1.0, 0.5, 0.5), |r| (r.zoom, r.center.x, r.center.y));
    json!({ "zoom": zoom, "centerX": x, "centerY": y }).to_string()
}

pub fn og_ui_progress(id: String, value: f64, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.progress", |ui| {
        ui.progress(
            id.as_str(),
            value as f32,
            ProgressDesc {
                width: size(&d["width"]),
            },
        )
    });
}

pub fn og_ui_slider(id: String, value: f64, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut v = value as f32;
    let r = with_ui("og.ui.slider", |ui| {
        ui.slider(
            id.as_str(),
            &mut v,
            SliderDesc {
                min: f(&d, "min", 0.0),
                max: f(&d, "max", 0.0),
                step: f(&d, "step", 0.0),
                width: size(&d["width"]),
            },
        )
    });
    json!({ "response": response(r.unwrap_or_default()), "value": v }).to_string()
}

pub fn og_ui_toggle(id: String, on: bool, label: String) -> String {
    let mut state = on;
    let r = with_ui("og.ui.toggle", |ui| {
        ui.toggle(id.as_str(), &mut state, &label)
    });
    json!({ "response": response(r.unwrap_or_default()), "on": state }).to_string()
}

pub fn og_ui_text_field(id: String, value: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut text = value;
    let r = with_ui("og.ui.textField", |ui| {
        let fd = FieldDesc {
            placeholder: d["placeholder"].as_str().unwrap_or_default().to_string(),
            width: size(&d["width"]),
            max_chars: d["maxChars"].as_u64().unwrap_or(0) as u32,
        };
        ui.text_field(id.as_str(), &mut text, &fd)
    });
    json!({ "response": response(r.unwrap_or_default()), "value": text }).to_string()
}

pub fn og_ui_text_area(id: String, value: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut text = value;
    let r = with_ui("og.ui.textArea", |ui| {
        let ad = og_ui::TextAreaDesc {
            placeholder: d["placeholder"].as_str().unwrap_or_default().to_string(),
            width: size(&d["width"]),
            rows: d["rows"].as_u64().unwrap_or(0) as u32,
            max_chars: d["maxChars"].as_u64().unwrap_or(0) as u32,
        };
        ui.text_area(id.as_str(), &mut text, &ad)
    });
    json!({ "response": response(r.unwrap_or_default()), "value": text }).to_string()
}

pub fn og_ui_counter(id: String, value: f64, desc: Option<String>) -> f64 {
    let d = desc_of(desc);
    with_ui("og.ui.counter", |ui| {
        let t = *ui.theme();
        let c = og_ui::CounterDesc {
            step: d["step"].as_str().and_then(|s| step(&t, s)),
            font: d["font"]
                .as_str()
                .and_then(FontRole::from_name)
                .unwrap_or(FontRole::Body),
            color: paint(&t, &d["color"]),
            align: match d["align"].as_str() {
                Some("center") => TextAlign::Center,
                Some("end") => TextAlign::End,
                _ => TextAlign::Start,
            },
            width: size_or(&d["width"], Size::fit()),
            duration_ms: f(&d, "durationMs", 0.0),
            from: d["from"].as_f64(),
            decimals: d["decimals"].as_u64().unwrap_or(0).min(9) as u32,
            prefix: d["prefix"].as_str().unwrap_or_default().to_string(),
            suffix: d["suffix"].as_str().unwrap_or_default().to_string(),
        };
        ui.counter(id.as_str(), value, &c)
    })
    .unwrap_or(value)
}

pub fn og_ui_burst(desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.burst", |ui| {
        let t = *ui.theme();
        ui.burst(og_ui::BurstDesc {
            count: d["count"].as_u64().unwrap_or(0).min(64) as u32,
            color: paint(&t, &d["color"]),
            radius: f(&d, "radius", 0.0),
            duration_ms: f(&d, "durationMs", 0.0),
            delay_ms: f(&d, "delayMs", 0.0),
        })
    });
}

pub fn og_ui_shake(desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.shake", |ui| {
        ui.shake(og_ui::ShakeDesc {
            amount: f(&d, "amount", 0.0),
            duration_ms: f(&d, "durationMs", 0.0),
            sound: d["sound"].as_bool().unwrap_or(false),
        })
    });
}

pub fn og_ui_screen_transition(id: String, screen: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let texture = d["texture"].as_f64().and_then(|h| {
        let i = super::assets::index_of(h)?;
        let target = BUILDING.with(Cell::get)?;
        let eng = engine(target).ok()?;

        eng.render.borrow().textures.get(i).copied()
    });
    let r = with_ui("og.ui.screenTransition", |ui| {
        let t = *ui.theme();
        let desc = og_ui::ScreenTransitionDesc {
            kind: match d["kind"].as_str() {
                Some("wipe") => og_ui::ScreenWipe::Wipe,
                Some("iris") => og_ui::ScreenWipe::Iris,
                Some("dissolve") => og_ui::ScreenWipe::Dissolve,
                Some("slide") => og_ui::ScreenWipe::Slide,
                _ => og_ui::ScreenWipe::Fade,
            },
            duration_ms: f(&d, "durationMs", 0.0),
            color: paint(&t, &d["color"]),
            texture: texture.map(|t| (t, [0.0, 0.0, 1.0, 1.0])),
            from: match d["from"].as_str() {
                Some("right") => og_ui::Side::Right,
                Some("top") => og_ui::Side::Top,
                Some("bottom") => og_ui::Side::Bottom,
                _ => og_ui::Side::Left,
            },
        };
        ui.screen_transition(id.as_str(), &screen, &desc)
    })
    .unwrap_or(og_ui::ScreenTransition {
        shown: screen.clone(),
        ..Default::default()
    });
    json!({ "shown": r.shown, "covering": r.covering, "revealing": r.revealing, "cover": r.cover })
        .to_string()
}

pub fn og_ui_tooltip(text: String) {
    with_ui("og.ui.tooltip", |ui| ui.tooltip(&text));
}

// ── ui: drawing over the frame ──────────────────────────────────────────────

/// Hands the UI's last frame to the render scene's overlay: screen-space
/// sprites and text in CSS pixels over the finished frame (after every post
/// effect, never tone mapped or blurred), at any camera.
pub(crate) fn emit(rs: &mut super::extract::RenderState) {
    let Some(ui) = rs.ui.clone() else {
        return;
    };
    let ui = ui.lock();
    let frame = ui.frame();
    if frame.items.is_empty() {
        return;
    }
    let atlas = *rs
        .ui_atlas
        .get_or_insert_with(|| rs.scene.add_texture(og_ui::atlas_texture()));
    let vp = frame.viewport;
    // Scale 0 fits the viewport's height to the target (as a capture at
    // another size needs).
    let overlay = rs
        .scene
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(vp.width.max(1.0), vp.height.max(1.0), 0.0));
    let first_text = overlay.texts.len();
    frame.emit(
        &og_ui::EmitDesc {
            atlas,
            // Over the game's screen layers up to og_render::UI_LAYER, under
            // those above it (og_render::screen_layer).
            layer: og_render::UI_OVERLAY_LAYER,
        },
        overlay,
    );
    // A multi-page font draws once per page with one layout.
    let mut more = Vec::new();
    for t in &overlay.texts[first_text..] {
        let pages = rs.ui_pages.get(&t.font).cloned().unwrap_or_default();
        for page in pages.iter().skip(1) {
            more.push(og_render::Text {
                font: *page,
                ..t.clone()
            });
        }
    }
    overlay.texts.extend(more);
}

// ── ui: the game widget set ─────────────────────────────────────────────────

thread_local! {
    /// Viewports set by og.ui.previewViewport, by engine: they win over the host's.
    static PREVIEW: std::cell::RefCell<std::collections::BTreeMap<u32, Viewport>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

fn opt_f(v: &Value, k: &str) -> Option<f32> {
    v[k].as_f64().map(|x| x as f32)
}

fn tone(v: &Value) -> Tone {
    match v.as_str() {
        Some("success") => Tone::Success,
        Some("warn") => Tone::Warn,
        Some("danger") => Tone::Danger,
        Some("info") => Tone::Info,
        Some("accent") => Tone::Accent,
        Some("inverse") => Tone::Inverse,
        _ => Tone::Neutral,
    }
}

fn anchor_of(v: &Value) -> Option<Anchor> {
    Some(match v.as_str()? {
        "topLeft" => Anchor::TopLeft,
        "top" => Anchor::Top,
        "topRight" => Anchor::TopRight,
        "left" => Anchor::Left,
        "center" => Anchor::Center,
        "right" => Anchor::Right,
        "bottomLeft" => Anchor::BottomLeft,
        "bottom" => Anchor::Bottom,
        "bottomRight" => Anchor::BottomRight,
        _ => return None,
    })
}

fn transition_kind(v: &Value) -> Option<TransitionKind> {
    Some(match v.as_str()? {
        "fade" => TransitionKind::Fade,
        "slideUp" => TransitionKind::SlideUp,
        "slideDown" => TransitionKind::SlideDown,
        "slideLeft" => TransitionKind::SlideLeft,
        "slideRight" => TransitionKind::SlideRight,
        "scale" => TransitionKind::Scale,
        "pop" => TransitionKind::Pop,
        _ => return None,
    })
}

fn strings(text: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(text).unwrap_or_default()
}

fn idx(v: &Value) -> Option<u32> {
    v.as_u64().map(|n| n as u32)
}

fn opt(v: Option<u32>) -> Value {
    v.map_or(Value::Null, |n| json!(n))
}

fn rect_json(r: &og_ui::Rect) -> Value {
    json!({ "x": r.x, "y": r.y, "width": r.width, "height": r.height })
}

fn texture_of(v: &Value) -> Option<og_render::TextureHandle> {
    let t = v.as_f64()?;
    let eng = engine(BUILDING.with(Cell::get).or_else(|| LAST.with(Cell::get))?).ok()?;
    let rs = eng.render.borrow();
    super::assets::index_of(t).and_then(|i| rs.textures.get(i).copied())
}

fn uv_of(v: &Value) -> Option<[f32; 4]> {
    v.as_array().filter(|a| a.len() == 4).map(|a| {
        let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
        [n(0), n(1), n(2), n(3)]
    })
}

pub fn og_ui_breakpoint() -> String {
    let b = with_ui("og.ui.breakpoint", |ui| ui.breakpoint()).unwrap_or_default();
    b.name().to_string()
}

pub fn og_ui_open_anchor(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.openAnchor", |ui| {
        let t = *ui.theme();
        let a = AnchorDesc {
            anchor: anchor_of(&d["anchor"]).unwrap_or_default(),
            margin: opt_f(&d, "margin"),
            offset: (f(&d, "offsetX", 0.0), f(&d, "offsetY", 0.0)),
            direction: match d["direction"].as_str() {
                Some("column") => Direction::Column,
                _ => Direction::Row,
            },
            gap: f(&d, "gap", 0.0),
            padding: insets(&d["padding"]),
            align: align(&d["align"]),
            cross: cross(&d["cross"]),
            background: paint(&t, &d["background"]),
            radius: f(&d, "radius", 0.0),
            width: size_or(&d["width"], Size::fit()),
            height: size_or(&d["height"], Size::fit()),
            over: d["over"].as_bool().unwrap_or(false),
            skin: skin_part(&d["frame"]),
        };
        ui.open_anchor(id.as_str(), a)
    });
}

fn presence_json(p: Presence) -> String {
    json!({ "visible": p.visible, "progress": p.progress, "entering": p.entering, "leaving": p.leaving }).to_string()
}

pub fn og_ui_open_transition(id: String, show: bool, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let p = with_ui("og.ui.openTransition", |ui| {
        ui.open_transition(
            id.as_str(),
            show,
            TransitionDesc {
                kind: transition_kind(&d["kind"]).unwrap_or_default(),
                duration_ms: f(&d, "durationMs", 0.0),
                distance: f(&d, "distance", 0.0),
                instant: d["instant"].as_bool().unwrap_or(false),
            },
        )
    });
    presence_json(p.unwrap_or_default())
}

pub fn og_ui_open_modal(id: String, open: bool, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.openModal", |ui| {
        ui.open_modal(
            id.as_str(),
            open,
            &ModalDesc {
                title: d["title"].as_str().unwrap_or_default().to_string(),
                width: size(&d["width"]),
                locked: d["locked"].as_bool().unwrap_or(false),
                ignore_backdrop: d["ignoreBackdrop"].as_bool().unwrap_or(false),
                transition: transition_kind(&d["transition"]),
                backdrop: paint(ui.theme(), &d["backdrop"]),
                bare: d["bare"].as_bool().unwrap_or(false),
            },
        )
    })
    .unwrap_or_default();
    json!({ "visible": r.visible, "dismissed": r.dismissed }).to_string()
}

pub fn og_ui_close_modal() {
    with_ui("og.ui.closeModal", |ui| ui.close_modal());
}

pub fn og_ui_spotlight(id: String, hole: String, desc: Option<String>) -> String {
    let h = parse(Some(&hole)).unwrap_or(Value::Null);
    let d = desc_of(desc);
    let hole = h.is_object().then(|| {
        og_ui::Rect::new(
            f(&h, "x", 0.0),
            f(&h, "y", 0.0),
            f(&h, "width", 0.0),
            f(&h, "height", 0.0),
        )
    });
    let r = with_ui("og.ui.spotlight", |ui| {
        ui.spotlight(
            id.as_str(),
            hole,
            SpotlightDesc {
                padding: f(&d, "padding", 0.0),
            },
        )
    });
    response(r.unwrap_or_default()).to_string()
}

pub fn og_ui_bounds(path: String) -> String {
    let b = with_ui("og.ui.bounds", |ui| {
        ui.frame()
            .element(&path)
            .map(|e| og_ui::Rect::new(e.x, e.y, e.width, e.height))
    })
    .flatten();
    b.map_or(Value::Null, |r| rect_json(&r)).to_string()
}

fn range_json(r: ListRange) -> String {
    json!({ "first": r.first, "last": r.last }).to_string()
}

pub fn og_ui_open_list(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.openList", |ui| {
        ui.open_list(
            id.as_str(),
            ListDesc {
                count: idx(&d["count"]).unwrap_or(0),
                row_height: f(&d, "rowHeight", 0.0),
                gap: f(&d, "gap", 0.0),
                width: size_or(&d["width"], Size::fit()),
                height: size(&d["height"]),
                padding: insets(&d["padding"]),
            },
        )
    });
    range_json(r.unwrap_or_default())
}

pub fn og_ui_open_grid(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.openGrid", |ui| {
        ui.open_grid(
            id.as_str(),
            GridDesc {
                count: idx(&d["count"]).unwrap_or(0),
                cell_width: f(&d, "cellWidth", 0.0),
                cell_height: f(&d, "cellHeight", 0.0),
                columns: idx(&d["columns"]).unwrap_or(0),
                gap: f(&d, "gap", 0.0),
                width: size_or(&d["width"], Size::fit()),
                height: size(&d["height"]),
                padding: insets(&d["padding"]),
            },
        )
    });
    range_json(r.unwrap_or_default())
}

pub fn og_ui_list_item(index: u32, selected: bool) -> String {
    response(with_ui("og.ui.listItem", |ui| ui.list_item(index, selected)).unwrap_or_default())
        .to_string()
}

pub fn og_ui_grid_item(index: u32, selected: bool) -> String {
    response(with_ui("og.ui.gridItem", |ui| ui.grid_item(index, selected)).unwrap_or_default())
        .to_string()
}

pub fn og_ui_close_list() -> String {
    response(with_ui("og.ui.closeList", |ui| ui.close_list()).unwrap_or_default()).to_string()
}

pub fn og_ui_dragging(id: String) -> String {
    opt(with_ui("og.ui.dragging", |ui| ui.dragging(id.as_str())).flatten()).to_string()
}

fn choice(r: Response, selected: usize) -> String {
    json!({ "response": response(r), "selected": selected }).to_string()
}

pub fn og_ui_tabs(id: String, items: String, selected: u32, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let items = strings(&items);
    let mut sel = selected as usize;
    let r = with_ui("og.ui.tabs", |ui| {
        let refs: Vec<&str> = items.iter().map(String::as_str).collect();
        ui.tabs(
            id.as_str(),
            &refs,
            &mut sel,
            TabsDesc {
                fill: d["fill"].as_bool().unwrap_or(false),
                width: size(&d["width"]),
            },
        )
    });
    choice(r.unwrap_or_default(), sel)
}

pub fn og_ui_select(id: String, options: String, selected: u32, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let options = strings(&options);
    let mut sel = selected as usize;
    let r = with_ui("og.ui.select", |ui| {
        let refs: Vec<&str> = options.iter().map(String::as_str).collect();
        ui.select(
            id.as_str(),
            &refs,
            &mut sel,
            &SelectDesc {
                placeholder: d["placeholder"].as_str().unwrap_or_default().to_string(),
                width: size(&d["width"]),
                disabled: d["disabled"].as_bool().unwrap_or(false),
            },
        )
    });
    choice(r.unwrap_or_default(), sel)
}

pub fn og_ui_stepper(id: String, value: f64, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let mut v = value as f32;
    let r = with_ui("og.ui.stepper", |ui| {
        ui.stepper(
            id.as_str(),
            &mut v,
            StepperDesc {
                min: f(&d, "min", 0.0),
                max: f(&d, "max", 0.0),
                step: f(&d, "step", 0.0),
                decimals: idx(&d["decimals"]).unwrap_or(0).min(6),
            },
        )
    });
    json!({ "response": response(r.unwrap_or_default()), "value": v }).to_string()
}

pub fn og_ui_radio(id: String, options: String, selected: u32, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let options = strings(&options);
    let mut sel = selected as usize;
    let r = with_ui("og.ui.radio", |ui| {
        let refs: Vec<&str> = options.iter().map(String::as_str).collect();
        ui.radio(
            id.as_str(),
            &refs,
            &mut sel,
            RadioDesc {
                horizontal: d["horizontal"].as_bool().unwrap_or(false),
            },
        )
    });
    choice(r.unwrap_or_default(), sel)
}

pub fn og_ui_checkbox(id: String, on: bool, label: String) -> String {
    let mut state = on;
    let r = with_ui("og.ui.checkbox", |ui| {
        ui.checkbox(id.as_str(), &mut state, &label)
    });
    json!({ "response": response(r.unwrap_or_default()), "on": state }).to_string()
}

pub fn og_ui_checkbox_list(id: String, labels: String, values: String) -> String {
    let labels = strings(&labels);
    let mut vals: Vec<bool> = serde_json::from_str(&values).unwrap_or_default();
    vals.resize(labels.len(), false);
    let r = with_ui("og.ui.checkboxList", |ui| {
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        ui.checkbox_list(id.as_str(), &refs, &mut vals)
    });
    json!({ "response": response(r.unwrap_or_default()), "values": vals }).to_string()
}

pub fn og_ui_chip(id: String, text: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.chip", |ui| {
        ui.chip(
            id.as_str(),
            &text,
            ChipDesc {
                selected: d["selected"].as_bool().unwrap_or(false),
                disabled: d["disabled"].as_bool().unwrap_or(false),
            },
        )
    });
    response(r.unwrap_or_default()).to_string()
}

pub fn og_ui_badge(id: String, text: String, t: String) {
    let tn = tone(&Value::String(t));
    with_ui("og.ui.badge", |ui| ui.badge(id.as_str(), &text, tn));
}

pub fn og_ui_divider(id: String, vertical: bool) {
    with_ui("og.ui.divider", |ui| ui.divider(id.as_str(), vertical));
}

pub fn og_ui_spacer(id: String, s: String) {
    let v = parse(Some(&s)).unwrap_or(Value::Null);
    with_ui("og.ui.spacer", |ui| ui.spacer(id.as_str(), size(&v)));
}

pub fn og_ui_key_value(id: String, key: String, value: String, mono: bool) {
    with_ui("og.ui.keyValue", |ui| {
        ui.key_value(id.as_str(), &key, &value, mono)
    });
}

pub fn og_ui_skeleton(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.skeleton", |ui| {
        ui.skeleton(
            id.as_str(),
            SkeletonDesc {
                width: size(&d["width"]),
                height: f(&d, "height", 0.0),
                lines: idx(&d["lines"]).unwrap_or(0),
            },
        )
    });
}

pub fn og_ui_spinner(id: String, s: f64) {
    with_ui("og.ui.spinner", |ui| ui.spinner(id.as_str(), s as f32));
}

pub fn og_ui_icon(id: String, name: String, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.icon", |ui| {
        let t = *ui.theme();
        ui.icon(
            id.as_str(),
            &name,
            IconDesc {
                size: f(&d, "size", 0.0),
                tint: paint(&t, &d["tint"]),
            },
        )
    });
}

pub fn og_ui_meter(id: String, value: f64, max: f64, desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.meter", |ui| {
        let t = *ui.theme();
        let kind = match (paint(&t, &d["color"]), d["kind"].as_str()) {
            (Some(c), _) => MeterKind::Custom(c),
            (None, Some("mana")) => MeterKind::Mana,
            (None, Some("xp")) => MeterKind::Xp,
            (None, Some("accent")) => MeterKind::Accent,
            _ => MeterKind::Health,
        };
        ui.meter(
            id.as_str(),
            value as f32,
            max as f32,
            &MeterDesc {
                kind,
                label: d["label"].as_str().unwrap_or_default().to_string(),
                width: size(&d["width"]),
                height: f(&d, "height", 0.0),
                segments: idx(&d["segments"]).unwrap_or(0),
            },
        )
    });
}

pub fn og_ui_cooldown(id: String, remaining: f64, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.cooldown", |ui| {
        ui.cooldown(
            id.as_str(),
            remaining as f32,
            &CooldownDesc {
                size: f(&d, "size", 0.0),
                label: d["label"].as_str().unwrap_or_default().to_string(),
                icon: d["icon"].as_str().unwrap_or_default().to_string(),
            },
        )
    });
    response(r.unwrap_or_default()).to_string()
}

pub fn og_ui_minimap(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    let texture = texture_of(&d["texture"]);
    with_ui("og.ui.minimap", |ui| {
        let markers = d["markers"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|m| MapMarker {
                        x: f(m, "x", 0.0),
                        y: f(m, "y", 0.0),
                        tone: tone(&m["tone"]),
                        player: m["player"].as_bool().unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default();
        ui.minimap(
            id.as_str(),
            &MinimapDesc {
                texture,
                uv: uv_of(&d["uv"]),
                size: f(&d, "size", 0.0),
                markers,
                label: d["label"].as_str().unwrap_or_default().to_string(),
            },
        )
    });
}

pub fn og_ui_dialogue(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let portrait = texture_of(&d["portrait"])
        .map(|t| (t, uv_of(&d["portraitUv"]).unwrap_or([0.0, 0.0, 1.0, 1.0])));
    let r = with_ui("og.ui.dialogue", |ui| {
        ui.dialogue(
            id.as_str(),
            &DialogueDesc {
                speaker: d["speaker"].as_str().unwrap_or_default().to_string(),
                text: d["text"].as_str().unwrap_or_default().to_string(),
                portrait,
                portrait_icon: d["portraitIcon"].as_str().unwrap_or_default().to_string(),
                chars_per_second: f(&d, "charsPerSecond", 0.0),
                portrait_right: d["portraitRight"].as_bool().unwrap_or(false),
            },
        )
    })
    .unwrap_or_default();
    json!({ "shown": r.shown, "done": r.done, "advanced": r.advanced }).to_string()
}

pub fn og_ui_joystick(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.joystick", |ui| {
        ui.joystick(
            id.as_str(),
            JoystickDesc {
                size: f(&d, "size", 0.0),
                dead_zone: f(&d, "deadZone", 0.0),
            },
        )
    })
    .unwrap_or_default();
    json!({ "x": r.x, "y": r.y, "active": r.active }).to_string()
}

pub fn og_ui_dpad(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.dpad", |ui| {
        ui.dpad(
            id.as_str(),
            DpadDesc {
                size: f(&d, "size", 0.0),
            },
        )
    })
    .unwrap_or_default();
    json!({ "up": r.up, "down": r.down, "left": r.left, "right": r.right }).to_string()
}

pub fn og_ui_action_button(id: String, text: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let r = with_ui("og.ui.actionButton", |ui| {
        ui.action_button(
            id.as_str(),
            &text,
            &ActionButtonDesc {
                size: f(&d, "size", 0.0),
                primary: d["primary"].as_bool().unwrap_or(false),
                icon: d["icon"].as_str().unwrap_or_default().to_string(),
            },
        )
    });
    response(r.unwrap_or_default()).to_string()
}

/// The UI of an engine, for calls that work between ticks too.
fn ui_of(world: u32) -> Option<SharedUi> {
    engine(world).ok()?.state.borrow().ui.clone()
}

pub fn og_ui_toast(world: u32, text: String, desc: Option<String>) -> u32 {
    let d = desc_of(desc);
    let Some(ui) = ui_of(world) else { return 0 };
    let mut ui = ui.lock();
    ui.toast(
        &text,
        ToastDesc {
            tone: tone(&d["tone"]),
            duration_ms: f(&d, "durationMs", 0.0),
        },
    )
}

pub fn og_ui_dismiss_toast(world: u32, toast_id: u32) {
    if let Some(ui) = ui_of(world) {
        ui.lock().dismiss_toast(toast_id);
    }
}

pub fn og_ui_navigate(world: u32, intent: String) {
    let nav = match intent.as_str() {
        "up" => Nav::Up,
        "down" => Nav::Down,
        "left" => Nav::Left,
        "right" => Nav::Right,
        "next" => Nav::Next,
        "prev" => Nav::Prev,
        "activate" => Nav::Activate,
        "back" => Nav::Back,
        _ => return,
    };
    if let Some(ui) = ui_of(world) {
        ui.lock().navigate(nav);
    }
}

pub fn og_ui_back() -> bool {
    with_ui("og.ui.back", |ui| ui.back()).unwrap_or(false)
}

pub fn og_ui_set_navigation(on: bool) {
    with_ui("og.ui.setNavigation", |ui| ui.set_navigation(on));
}

pub fn og_ui_set_players(count: u32) {
    with_ui("og.ui.setPlayers", |ui| ui.set_players(count));
}

pub fn og_ui_focus(pad: u32) -> String {
    with_ui("og.ui.focus", |ui| ui.focus_path(pad).to_string()).unwrap_or_default()
}

/// An accessibility tree as og.ui.a11y answers it (IDL `ui.A11yTree`).
pub(crate) fn a11y_json(t: &og_ui::A11yTree) -> Value {
    let nodes: Vec<Value> = t
        .nodes
        .iter()
        .map(|n| {
            json!({
                "path": n.path, "parent": n.parent, "role": n.role, "name": n.name,
                "description": n.description, "value": n.value, "states": n.states,
                "x": n.x, "y": n.y, "width": n.width, "height": n.height,
            })
        })
        .collect();
    json!({"nodes": nodes, "focusOrder": t.focus_order, "focused": t.focused})
}

pub fn og_ui_a11y() -> String {
    with_ui("og.ui.a11y", |ui| a11y_json(ui.a11y()).to_string())
        .unwrap_or_else(|| r#"{"nodes":[],"focusOrder":[],"focused":""}"#.to_string())
}

pub fn og_ui_memory(id: String, initial: f64) -> f64 {
    with_ui("og.ui.memory", |ui| ui.memory(&id, initial)).unwrap_or(initial)
}

pub fn og_ui_remember(id: String, value: f64) {
    with_ui("og.ui.remember", |ui| ui.remember(&id, value));
}

pub fn og_ui_tick() -> u32 {
    with_ui("og.ui.tick", |ui| ui.tick() as u32).unwrap_or(0)
}

pub fn og_ui_preview_viewport(world: u32, viewport: String) {
    let v = parse(Some(&viewport)).unwrap_or(Value::Null);
    PREVIEW.with(|p| {
        let mut p = p.borrow_mut();
        if v.is_object() {
            let num = |k: &str, d: f64| v[k].as_f64().unwrap_or(d) as f32;
            p.insert(
                world,
                Viewport {
                    width: num("width", 1280.0).max(1.0),
                    height: num("height", 720.0).max(1.0),
                    dpr: num("dpr", 1.0),
                    safe_area: insets(&v["safeArea"]),
                    coarse_pointer: v["coarsePointer"].as_bool().unwrap_or(false),
                    reduced_motion: v["reducedMotion"].as_bool().unwrap_or(false),
                    high_contrast: v["highContrast"].as_bool().unwrap_or(false),
                    reduced_transparency: v["reducedTransparency"].as_bool().unwrap_or(false),
                    mac_keys: v["macKeys"].as_bool().unwrap_or(false),
                },
            );
        } else {
            p.remove(&world);
        }
    });
}

/// The previewed viewport of an engine, when one is set.
fn preview_of(engine_id: u32) -> Option<Viewport> {
    PREVIEW.with(|p| p.borrow().get(&engine_id).copied())
}
