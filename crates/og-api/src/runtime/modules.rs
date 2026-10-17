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
//! 6. `og.audio`: turns the tick's AudioSource changes into commands.
//!
//! The UI's interaction state is the `og.ui` state block, so snapshots and
//! the world hash carry it with the physics world (`og.physics`) and the
//! audio stream (its resources).

use super::engine::{engine, running, Engine};
use super::{ok, ok_void, parse, refuse};
use og_core::glam::Vec3;
use og_ecs::{Entity, StateBlock, SystemDesc, World};
use og_physics::{
    MeshDesc, OverlapDesc, Physics, PhysicsQuery, PhysicsSystem, QueryFilter, RaycastDesc,
    ShapeCastDesc, ShapeDesc,
};
use og_ui::{
    Align, ButtonDesc, ButtonVariant, ControlSize, Cross, DensityMode, Direction, Element,
    FieldDesc, FontRole, Fonts, ImageDesc, InputFrame, Insets, LabelDesc, PanelDesc, ProgressDesc,
    Response, Scheme, ScrollDesc, SharedUi, Size, SliderDesc, Srgba, TextAlign, Theme, TypeStep,
    Ui, UiDesc, UiFont, Viewport,
};
use serde_json::{json, Map, Value};
use std::any::Any;
use std::cell::Cell;

/// The system that opens the UI frame, before the game's systems.
pub const UI_BEGIN: &str = "og.ui.begin";
/// The system that closes and lays out the UI frame, after them.
pub const UI_END: &str = "og.ui.end";
/// The state block that carries the UI's interaction state.
pub const UI_STATE: &str = "og.ui";
/// The sprite layer the UI's painter level 0 draws on: above game sprites.
pub const UI_LAYER: i32 = 1 << 20;

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
    let (theme, fonts) = match previous {
        Some(p) => {
            let ui = p.lock();
            (*ui.theme(), ui.fonts().clone())
        }
        None => (Theme::default(), Fonts::default()),
    };
    SharedUi::new(Ui::new(UiDesc {
        theme,
        ticks_per_second: tick,
        fonts,
    }))
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
            SystemDesc::new(UI_BEGIN).read::<og_probe::Input>(),
            move |ctx| {
                let input = ctx
                    .resource::<og_probe::Input>()
                    .map(|i| InputFrame::read(&*i))
                    .unwrap_or_default();
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
    physics.schedule(world).map_err(err)?;
    let audio = og_audio::install(
        world,
        &og_audio::AudioDesc {
            link,
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
/// safe area and the pointer kind. The web host calls it when the canvas
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
    };
    eng.render.borrow_mut().viewport = Some(v);
    Ok(())
}

fn viewport_json(v: &Viewport) -> Value {
    json!({
        "width": v.width, "height": v.height, "dpr": v.dpr,
        "safeArea": { "left": v.safe_area.left, "top": v.safe_area.top, "right": v.safe_area.right, "bottom": v.safe_area.bottom },
        "coarsePointer": v.coarse_pointer,
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
        interactive: d["interactive"].as_bool().unwrap_or(false),
        background: paint(theme, &d["background"]),
        radius: f(d, "radius", 0.0),
    }
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
            let text = format!("{call} was called outside a system: og.ui calls build the frame the engine opens before the game's systems and closes after them, so they do nothing anywhere else");
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

pub fn og_ui_configure(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let mut theme = match d["scheme"].as_str() {
            Some("light") => Theme::light(),
            _ => Theme::dark(),
        };
        theme.density = match d["density"].as_str() {
            Some("compact") => DensityMode::Compact,
            Some("comfortable") => DensityMode::Comfortable,
            Some("touch") => DensityMode::Touch,
            _ => DensityMode::Auto,
        };
        let font = |key: &str| -> Result<Option<UiFont>, String> {
            match d[key].as_f64() {
                None => Ok(None),
                Some(h) => ui_font(&eng, h).map(Some),
            }
        };
        let body = font("font")?;
        let fonts = Fonts {
            body: body.unwrap_or_default(),
            display: font("displayFont")?,
            mono: font("monoFont")?,
        };
        let ui = eng
            .state
            .borrow()
            .ui
            .clone()
            .ok_or("this engine has no UI")?;
        let mut ui = ui.lock();
        ui.set_theme(theme);
        ui.set_fonts(fonts);
        Ok(())
    })();
    match answer {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// A loaded font as the UI lays out and draws with it: its first atlas
/// page's handle and metrics (every page carries every glyph's advance);
/// the renderer draws the other pages from the same layout.
fn ui_font(eng: &Engine, handle: f64) -> Result<UiFont, String> {
    let index = super::assets::index_of(handle).ok_or_else(|| format!("{handle} is not a font"))?;
    let render = eng
        .assets
        .borrow()
        .fonts
        .get(index)
        .map(|f| f.render)
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
    Ok(UiFont::new(first, &msdf))
}

pub fn og_ui_viewport(world: u32) -> String {
    viewport_json(&viewport_of(world)).to_string()
}

pub fn og_ui_theme(world: u32) -> String {
    let (theme, sizes) = engine(world)
        .ok()
        .and_then(|e| e.state.borrow().ui.clone())
        .map(|ui| {
            let ui = ui.lock();
            (*ui.theme(), ui.sizes())
        })
        .unwrap_or_else(|| {
            let t = Theme::default();
            let s = t.sizes(&Viewport::default());
            (t, s)
        });
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
            font: match d["font"].as_str() {
                Some("display") => FontRole::Display,
                Some("mono") => FontRole::Mono,
                _ => FontRole::Body,
            },
        };
        ui.label(id.as_str(), &text, l)
    });
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
        };
        ui.image(id.as_str(), i)
    });
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

pub fn og_ui_tooltip(text: String) {
    with_ui("og.ui.tooltip", |ui| ui.tooltip(&text));
}

// ── ui: drawing over the frame ──────────────────────────────────────────────

/// Appends the UI's last frame to the render scene, placed in front of the
/// scene's camera so its CSS pixels land where the probe and the pointer
/// see them. og-render has no screen-space overlay yet (og-ui README
/// §Drawing), so the quads lie on a plane `depth` in front of the camera,
/// which works for a camera looking down -Z (any roll; a 2D game's
/// orthographic camera, or a 3D camera without pitch or yaw). For any other
/// camera the UI is not drawn and `false` says so; it still answers the
/// probe and the pointer.
pub(crate) fn emit(rs: &mut super::extract::RenderState) -> bool {
    let Some(ui) = rs.ui.clone() else {
        return true;
    };
    let ui = ui.lock();
    let frame = ui.frame();
    if frame.items.is_empty() {
        return true;
    }
    let atlas = *rs
        .ui_atlas
        .get_or_insert_with(|| rs.scene.add_texture(og_ui::atlas_texture()));
    let cam = rs.scene.camera;
    let forward = (cam.target - cam.position).normalize_or_zero();
    if (forward - Vec3::NEG_Z).length() > 1e-3 {
        return false;
    }
    let vp = frame.viewport;
    let (w, h) = (vp.width.max(1.0), vp.height.max(1.0));
    let depth = (cam.near * 1.5).max(1e-3);
    let half_h = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => {
            depth * (fov_y_degrees.to_radians() / 2.0).tan()
        }
        og_render::Projection::Orthographic { height } => height / 2.0,
    };
    let k = 2.0 * half_h / h;
    let up = Vec3::new(cam.up.x, cam.up.y, 0.0).normalize_or(Vec3::Y);
    let right = Vec3::new(up.y, -up.x, 0.0);
    let roll = (-up.x).atan2(up.y);
    let origin = cam.position + Vec3::NEG_Z * depth;
    // UI world space: CSS (x, y) is (x, -y), so (ux - w/2, uy + h/2) is the
    // offset from the surface's centre, y up.
    let place = |p: Vec3| origin + right * ((p.x - w / 2.0) * k) + up * ((p.y + h / 2.0) * k);
    let mut sprites = Vec::new();
    let mut texts = Vec::new();
    frame.emit(
        &og_ui::EmitDesc {
            atlas,
            layer: UI_LAYER,
            z: 0.0,
        },
        &mut sprites,
        &mut texts,
    );
    for mut s in sprites {
        s.position = place(s.position);
        s.size *= k;
        s.rotation += roll;
        rs.scene.sprites.push(s);
    }
    for mut t in texts {
        t.position = place(t.position);
        t.size *= k;
        t.rotation += roll;
        let pages = rs.ui_pages.get(&t.font).cloned().unwrap_or_default();
        for page in pages.iter().skip(1) {
            rs.scene.texts.push(og_render::Text {
                font: *page,
                ..t.clone()
            });
        }
        rs.scene.texts.push(t);
    }
    true
}
