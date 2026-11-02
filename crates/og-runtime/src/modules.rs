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

mod ui_config;
pub(crate) use ui_config::{LoadedStyle, load_style, new_ui, ui_font};
pub use ui_config::{og_load_style, og_ui_configure};
mod ui_desc;
pub use ui_desc::{
    THEME_COLORS, TYPE_STEPS, og_ui_frame_direction, og_ui_frame_pointer, og_ui_frame_theme,
    og_ui_frame_viewport, og_ui_has_icon, og_ui_last_path, og_ui_scroll_into_view, og_ui_seconds,
    og_ui_set_focus, og_ui_theme, og_ui_viewport,
};
use ui_desc::{
    align, cross, element, f, frame_part, idx, insets, opt, opt_f, paint, rect_json, response,
    size, size_or, strings, tone, transition_kind, ui_rect, uv_of,
};
pub(crate) use ui_desc::{anchor_of, step, texture_of, ui_engine, with_ui};
mod widgets;
use widgets::desc_of;
pub(crate) use widgets::emit;
pub use widgets::{
    og_ui_burst, og_ui_button, og_ui_close, og_ui_counter, og_ui_image, og_ui_label, og_ui_open,
    og_ui_open_panel, og_ui_open_scroll, og_ui_progress, og_ui_rich_text, og_ui_screen_transition,
    og_ui_shake, og_ui_slider, og_ui_text_area, og_ui_text_field, og_ui_toggle, og_ui_tooltip,
    og_ui_zoom_view,
};
mod game_widgets;
use crate::{art, assets, extract, ui_draw};
pub(crate) use game_widgets::a11y_json;
use game_widgets::ui_of;
pub use game_widgets::{
    og_ui_a11y, og_ui_action_button, og_ui_back, og_ui_badge, og_ui_bounds, og_ui_breakpoint,
    og_ui_checkbox, og_ui_checkbox_list, og_ui_chip, og_ui_close_list, og_ui_close_modal,
    og_ui_cooldown, og_ui_dialogue, og_ui_dismiss_toast, og_ui_divider, og_ui_dpad, og_ui_dragging,
    og_ui_feedback, og_ui_focus, og_ui_grid_item, og_ui_icon, og_ui_joystick, og_ui_key_value,
    og_ui_list_item, og_ui_memory, og_ui_meter, og_ui_minimap, og_ui_navigate, og_ui_open_anchor,
    og_ui_open_grid, og_ui_open_list, og_ui_open_modal, og_ui_open_transition, og_ui_part_attr,
    og_ui_preview_viewport, og_ui_radio, og_ui_remember, og_ui_select, og_ui_set_haptics,
    og_ui_set_navigation, og_ui_set_players, og_ui_skeleton, og_ui_spacer, og_ui_spinner,
    og_ui_spotlight, og_ui_stepper, og_ui_tabs, og_ui_tick, og_ui_toast,
};

pub(crate) mod docs;
pub use docs::{
    og_load_document, og_ui_action, og_ui_action_bar, og_ui_action_held, og_ui_clear,
    og_ui_component, og_ui_define_action, og_ui_device, og_ui_doc_events, og_ui_document_schema,
    og_ui_frame_glyph, og_ui_game_input, og_ui_glyph, og_ui_layers, og_ui_open_slot, og_ui_paused,
    og_ui_pop, og_ui_push, og_ui_replace, og_ui_save_edits, og_ui_screen, og_ui_screen_vm,
    og_ui_set_device, og_ui_set_prop,
};
mod motion;
/// og.ui's world-space members (docs/ui.md §14).
pub(crate) mod ui_world;
pub use motion::{
    og_ui_animate, og_ui_define_timeline, og_ui_force_state, og_ui_gesture, og_ui_layout_stats,
    og_ui_motion_events, og_ui_open_path, og_ui_overlaps, og_ui_pause, og_ui_play, og_ui_play_info,
    og_ui_play_timeline, og_ui_props, og_ui_resume, og_ui_reverse, og_ui_seek, og_ui_semantics,
    og_ui_stop, og_ui_visual_state,
};

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
    FontRole, Fonts, GridDesc, IconAtlas, IconDesc, ImageDesc, InputFrame, Insets, JoystickDesc,
    LabelDesc, ListDesc, ListRange, MapMarker, MeterDesc, MeterKind, MinimapDesc, ModalDesc, Nav,
    PanelDesc, Presence, ProgressDesc, RadioDesc, Response, Scheme, ScrollDesc, SelectDesc,
    SharedUi, Size, SkeletonDesc, SliderDesc, SpotlightDesc, Srgba, StepperDesc, StylePart,
    TabsDesc, TextAlign, Theme, ToastDesc, Tone, TransitionDesc, TransitionKind, TypeStep, Ui,
    UiDesc, UiFont, Viewport,
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
    /// The world's gameplay actions (og-input's action map, og.input.*).
    pub actions: og_input::SharedActions,
}

/// The modules' systems that run before the game's: physics' collider
/// recording and the UI's begin. Registers physics' types and settings.
pub(crate) fn mount_front(
    world: &mut World,
    engine_id: u32,
    profile: u32,
    ui: &SharedUi,
) -> Result<(PhysicsSystem, og_input::SharedActions), String> {
    let physics = PhysicsSystem::register(
        world,
        Physics {
            profile,
            ..Physics::default()
        },
    )
    .map_err(err)?;
    // The 2D profile has no terrain, so no height field (terrain_absent.rs).
    #[cfg(feature = "profile2d")]
    physics.refuse_heightfields(super::terrain::ABSENT);
    physics.set_mesh_source(Some(std::sync::Arc::new(move |h| {
        geometry_mesh(engine_id, h)
    })));
    let (desc, run) = physics.query_system();
    world.add_system(desc, run).map_err(err)?;
    // Gameplay actions, read from the tick's input before the UI and the
    // game's systems (og-input's actions.rs); touch zones are fractions of
    // the play surface the UI lays out in.
    let actions: og_input::SharedActions = Default::default();
    og_input::install(world, actions.clone(), move || {
        let v = viewport_of(engine_id);
        Some(og_input::Surface {
            width: v.width,
            height: v.height,
            coarse: v.coarse_pointer,
        })
    })
    .map_err(err)?;
    let feed = actions.clone();
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
                    .with_a11y(&asked)
                    .with_actions(og_input::actions::lock(&feed).feed());
                let viewport = viewport_of(engine_id);
                shared.lock().begin(input, viewport);
                BUILDING.with(|b| b.set(Some(engine_id)));
                LAST.with(|l| l.set(Some(engine_id)));
                Ok(())
            },
        )
        .map_err(err)?;
    Ok((physics, actions))
}

/// The modules' systems that run after the game's: the UI's end, the
/// physics step and audio; then the UI's state block.
pub(crate) fn mount_back(
    world: &mut World,
    engine_id: u32,
    physics: PhysicsSystem,
    ui: SharedUi,
    link: Option<og_audio::AudioLink>,
    actions: og_input::SharedActions,
) -> Result<Mounted, String> {
    let shared = ui.clone();
    let feed = ui_world::Feed::of(world);
    world
        .add_system(feed.desc(UI_END), move |ctx| {
            let warnings = {
                let mut ui = shared.lock();
                docs::end_of_frame(engine_id, &mut ui, ctx);
                // The tick's camera and anchored entities (docs/ui.md §14).
                feed.before_end(ctx, &mut ui);
                BUILDING.with(|b| b.set(None));
                ui.end().warnings.clone()
            };
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
                // Each play is one of its event's set, at the volume and
                // pitch the set's ranges gave it (og-ui chose them from
                // the UI state's own stream, so a replay plays the same).
                let plays = shared.lock().take_feedback();
                if plays.is_empty() {
                    return Ok(());
                }
                for play in plays {
                    if play.sound == 0 {
                        continue;
                    }
                    let slot = play.event.index();
                    let old = shared.lock().voices_mut()[slot];
                    if old != 0 {
                        let e = Entity::from_bits(old);
                        if ctx.is_alive(e) {
                            ctx.despawn(e)?;
                        }
                    }
                    let e = ctx.spawn((og_audio::AudioSource {
                        bus: og_audio::Bus::Ui as u32,
                        volume: play.volume,
                        pitch: play.pitch,
                        ..og_audio::AudioSource::new(play.sound)
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
    // The sounds' QOA frames are decoded beside the tick: natively on the
    // bank's own thread, on the web by the host's decoder Worker
    // (og-audio's `decode.rs`). Once per bank: a reload keeps the old one.
    #[cfg(not(target_arch = "wasm32"))]
    audio.bank().start_decode_thread();
    world
        .add_state_block(UI_STATE, Box::new(UiState(ui.clone())))
        .map_err(err)?;
    world
        .add_state_block(
            ui_world::UI_WORLD_STATE,
            Box::new(ui_world::WorldState(ui.clone())),
        )
        .map_err(err)?;
    Ok(Mounted {
        physics,
        ui,
        audio,
        actions,
    })
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
/// pixel, else 1280×720; a native host (which sets none) on macOS edits
/// text with the Mac's keys ([`native_keys`]). A `cover` command's
/// rectangle ([`set_cover`]) is its `covered`, whichever of these it is.
pub(crate) fn viewport_of(engine_id: u32) -> Viewport {
    let Ok(eng) = engine(engine_id) else {
        return preview_of(engine_id).unwrap_or_else(|| native_keys(Viewport::default()));
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return preview_of(engine_id).unwrap_or_else(|| native_keys(Viewport::default()));
    };
    let mut v = preview_of(engine_id).or(rs.viewport).unwrap_or_else(|| {
        native_keys(match rs.backend.as_ref().map(|b| b.size()) {
            Some((w, h)) if w > 0 && h > 0 => Viewport::new(w as f32, h as f32),
            _ => Viewport::default(),
        })
    });
    if let Some(c) = rs.cover {
        v.covered = c;
    }
    v
}

/// The command channel's `cover:<x>,<y>,<width>,<height>` (CSS pixels) or
/// `cover:none`: where a host's own control covers the play surface
/// (`Viewport.covered`), kept over whatever the host reports until
/// cleared. A native run has no shell, so a playtest the published page
/// is held to covers the page's Back (12, 12, 40 by 40 on a desktop page)
/// and its frames lay out as the page's do.
// A publish build registers no command, so nothing reaches it there.
#[cfg_attr(feature = "publish", allow(dead_code))]
pub(crate) fn set_cover(engine_id: u32, args: &[String]) -> Result<(), String> {
    let arg = args
        .first()
        .filter(|a| !a.is_empty())
        .ok_or("write cover:<x>,<y>,<width>,<height> (CSS pixels, such as cover:12,12,40,40) or cover:none")?;
    let cover = if arg == "none" {
        None
    } else {
        let n: Vec<f32> = arg
            .split(',')
            .map(|s| s.trim().parse::<f32>().ok().filter(|x| x.is_finite() && *x >= 0.0))
            .collect::<Option<_>>()
            .filter(|n: &Vec<f32>| n.len() == 4)
            .ok_or_else(|| format!("cover:{arg}: write four numbers, x,y,width,height in CSS pixels (such as 12,12,40,40), or none"))?;
        Some(og_ui::Rect::new(n[0], n[1], n[2], n[3]))
    };
    engine(engine_id)?.render.borrow_mut().cover = cover;
    Ok(())
}

/// A native host's viewport takes the platform's editing keys: on macOS
/// Command takes the shortcuts and Option the words, as a Mac text field
/// does (`Viewport.macKeys`); elsewhere Control. The web host says which
/// with [`set_viewport`] (the browser's platform), so this is the native
/// hosts' alone.
pub(crate) fn native_keys(v: Viewport) -> Viewport {
    Viewport {
        mac_keys: cfg!(target_os = "macos"),
        ..v
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
        // Absent, the platform's own keys (the web host always says; a
        // native host's surface without a renderer, og-host-node's
        // og_node_attach_offscreen, leaves it to the platform, as
        // `native_keys` does for the rendered one).
        mac_keys: viewport["macKeys"]
            .as_bool()
            .unwrap_or(cfg!(target_os = "macos")),
        covered: ui_rect(&viewport["covered"]),
    };
    eng.render.borrow_mut().viewport = Some(v);
    Ok(())
}

/// A viewport as og.ui.viewport answers it, with the HUD band the UI's
/// `hud_max_aspect` makes of it (og-ui's `Ui::hud_area`).
fn viewport_json(v: &Viewport, hud_max_aspect: f32) -> Value {
    json!({
        "hud": rect_json(&v.hud_rect(hud_max_aspect)),
        "width": v.width, "height": v.height, "dpr": v.dpr,
        "safeArea": { "left": v.safe_area.left, "top": v.safe_area.top, "right": v.safe_area.right, "bottom": v.safe_area.bottom },
        "coarsePointer": v.coarse_pointer,
        "reducedMotion": v.reduced_motion,
        "highContrast": v.high_contrast,
        "reducedTransparency": v.reduced_transparency,
        "macKeys": v.mac_keys,
        "covered": rect_json(&v.covered),
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
        Ok(hit.map_or(Value::Null, |h| hit_json(&h)))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn hit_json(h: &og_physics::RaycastHit) -> Value {
    json!({
        "entity": h.entity.to_bits(), "body": h.body.to_bits(), "distance": h.distance,
        "point": v3(h.point), "normal": v3(h.normal),
    })
}

pub fn og_physics_raycast_all(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let d = parse(desc.as_deref())?;
        let ray = RaycastDesc {
            origin: vec3(&d["origin"], Vec3::ZERO),
            direction: vec3(&d["direction"], Vec3::NEG_Y),
            max_distance: d["maxDistance"].as_f64().unwrap_or(0.0) as f32,
            filter: filter(&d["filter"])?,
        };
        let max_hits = d["maxHits"].as_u64().unwrap_or(0).min(u64::from(u32::MAX)) as u32;
        let hits = query(world)?
            .raycast_all(&og_physics::RaycastAllDesc { ray, max_hits })
            .map_err(err)?;
        Ok(Value::Array(hits.iter().map(hit_json).collect()))
    })();
    match answer {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_physics_project_point(world: u32, desc: Option<String>) -> String {
    let answer = (|| -> Result<_, String> {
        let d = parse(desc.as_deref())?;
        let desc = og_physics::ProjectPointDesc {
            point: vec3(&d["point"], Vec3::ZERO),
            max_distance: d["maxDistance"].as_f64().unwrap_or(0.0) as f32,
            solid: d["solid"].as_bool().unwrap_or(true),
            filter: filter(&d["filter"])?,
        };
        let p = query(world)?.project_point(&desc).map_err(err)?;
        Ok(p.map_or(Value::Null, |p| {
            json!({
                "entity": p.entity.to_bits(), "body": p.body.to_bits(), "point": v3(p.point),
                "normal": v3(p.normal), "distance": p.distance, "inside": p.inside,
            })
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

/// Geometry `handle` of engine `engine_id` as a physics mesh (physics'
/// mesh source: a collider naming a geometry nobody registered). None from
/// another thread than the engine's, or while its renderer is borrowed.
fn geometry_mesh(engine_id: u32, handle: u64) -> Option<MeshDesc> {
    let eng = engine(engine_id).ok()?;
    let rs = eng.render.try_borrow().ok()?;
    let m = *rs.geometries.get(super::assets::index_of(handle as f64)?)?;
    Some(mesh_of(rs.scene.mesh(m)?))
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

// ── ui: the game widget set ─────────────────────────────────────────────────

thread_local! {
    /// Viewports set by og.ui.previewViewport, by engine: they win over the host's.
    static PREVIEW: std::cell::RefCell<og_core::IdMap<u32, Viewport>> =
        const { std::cell::RefCell::new(og_core::IdMap::new()) };
}

/// The previewed viewport of an engine, when one is set.
fn preview_of(engine_id: u32) -> Option<Viewport> {
    PREVIEW.with(|p| p.borrow().get(&engine_id).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A native host sets no viewport: on macOS its text controls take the
    /// Mac's keys (Command for the shortcuts, Option for words), elsewhere
    /// Control; one the host set (the web shell's) is kept as it is.
    #[test]
    fn a_native_host_edits_with_the_platforms_keys() {
        let v: Value = serde_json::from_str(&og_ui_viewport(u32::MAX)).expect("json");
        assert_eq!(
            v["macKeys"].as_bool(),
            Some(cfg!(target_os = "macos")),
            "{v}"
        );
        let set = Viewport {
            mac_keys: false,
            ..Viewport::new(800.0, 600.0)
        };
        PREVIEW.with(|p| p.borrow_mut().insert(u32::MAX - 1, set));
        assert!(
            !viewport_of(u32::MAX - 1).mac_keys,
            "a set viewport is kept"
        );
    }
}
