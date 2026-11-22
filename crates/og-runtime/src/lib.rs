//! Origin Engine `og-runtime`: the host-neutral runtime both hosts wrap.
//!
//! `og-host-web` and `og-host-node` implement the generated `og_api::host`
//! traits; almost every method is a call into this module with the same wire
//! arguments. What differs per host stays in the host: how column memory is
//! handed to JavaScript, how the dispatcher is called, and the renderer's
//! surface. The model:
//!
//! - **Declarations** (`defineComponent`, `defineResource`, `defineSystem`,
//!   `og.game`, `og.commands.register`) are recorded per thread in call
//!   order, before any world exists, exactly as the bundle makes them.
//! - **Engines** (`og_engine_create`) assemble an `og_ecs::World` from a game
//!   and every declaration: engine-owned records first (in IDL order), then
//!   the game's components and resources (in declaration order), then its
//!   systems (declaration order, then `after`/`before`). The order is a
//!   function of the bundle, so both hosts build the same world.
//! - **Systems** are registered with `og-ecs` as one closure each that calls
//!   the host's dispatcher with the system's callback id (`Fn + Send`, so it
//!   captures ids only). While a system runs, its columns are borrowed and
//!   published as views; the JavaScript body calls back into the host, which
//!   serves those calls from the running system's context.
//! - **Views** are stamped: a view handle is valid for the system call (or,
//!   outside systems, until the next structural change) it was made in, and
//!   the host refuses it after. The TypeScript runtime throws on a stale view
//!   with its tick and system.
//!
//! Single-threaded by construction: an engine never leaves the thread that
//! created it. What an engine owns is on the engine (its world, its
//! modules' state, its caches); only the process roots are thread-local
//! (the declarations, the engines' table, the handles, the installed hooks
//! and the call on the stack; `engine.rs` says why each stays).
//!
//! The sources, by what they answer to:
//!
//! - `engine/`: declarations, assembly, the tick, the dispatcher, the view
//!   tables, each engine's module state. Assembly mounts every module by
//!   name: the order of its calls is the schedule.
//! - `host/`: the boundary both hosts forward to, its wire types and
//!   layouts, and what only a host can answer.
//! - `bind/`: one binding per domain crate over the runtime's engines, the
//!   stub of each optional one, and the wiring where two domains meet.
//! - `world/`: the world's content and its persistence (loads, prefabs,
//!   streaming, the moving origin, saves).
//! - `present/`: what a frame shows: extraction, interpolation, the
//!   presentation checkpoint, looks, effects and game feel.
//! - `author/`: what an author and an agent see of a running engine: the
//!   probe and its views, the inspector, traces, recorded frames.

#![recursion_limit = "256"]

mod author;
mod bind;
mod engine;
mod host;
mod present;
mod registry;
mod world;

// The hosts' paths into the boundary.
pub use host::{host_impl, shared, ui_cursor, wire};

pub use author::lanes::{panic_report, record_panic, report as lanes_report};

pub use author::memory::memory_report;
pub use author::perf::{report as perf_report, set as perf_set, take_frames as perf_frames};
#[cfg(not(feature = "publish"))]
pub use author::probe::dev_refresh;
#[cfg(not(feature = "publish"))]
pub use author::record::{render_frame, render_restore, render_snapshot};
pub use author::trace::{
    BEFORE_ENGINE as TRACE_BEFORE_ENGINE, set as trace_set, take as trace_take,
};
pub use bind::exports::*;
pub use engine::invariants::og_assert_record;
pub use engine::{
    ColumnPtr, Dispatch, Hooks, RenderBackend, TextureWork, VideoDecoder, advance, answer_call,
    antialias_name, begin_bundle, call_rpc, command, create, destroy, frame, point_shadows_name,
    render_desc, render_options, set_render_backend, take_log, take_outputs, transparency_name,
    view_add_scaled, view_columns, view_entities, warm_renderer, write_resource,
};
pub use host::api::*;
pub use host::codec::{Kind, Layout, RecordKind};
pub use host::platform::{
    og_platform_achievement_progress, og_platform_achievement_unlock, og_platform_achievements,
    og_platform_ad_ready, og_platform_ai_mark, og_platform_answer, og_platform_catalog_head,
    og_platform_consume, og_platform_entitlements, og_platform_friends, og_platform_grant,
    og_platform_granted, og_platform_info, og_platform_invite_send, og_platform_invites,
    og_platform_journal, og_platform_leaderboard_around, og_platform_leaderboard_submit,
    og_platform_leaderboard_top, og_platform_load_settings, og_platform_load_slot,
    og_platform_notice, og_platform_notify, og_platform_now, og_platform_player, og_platform_point,
    og_platform_products, og_platform_purchase, og_platform_result, og_platform_save_settings,
    og_platform_save_slot, og_platform_set_host, og_platform_share, og_platform_show_ad,
    og_platform_store_get, og_platform_store_list, og_platform_store_set, og_platform_supports,
    og_platform_take, og_platform_telemetry,
};
/// The counting allocator the hosts install (`#[global_allocator]`), for the probe's `memory`.
pub use og_core::alloc::Counting as CountingAlloc;
pub use og_render::png;
pub use present::atmosphere::{
    og_atmosphere_time_of_day, og_load_shader, og_render_set_material_shader,
    og_render_set_shader_param, og_render_shader_material,
};
pub use present::baked::og_render_baked_lighting;
pub use present::display::{
    FrameKind, frames_in_flight, lose_device, og_render_display, og_render_reset_display,
    og_render_set_display, set_capture_only, set_frame_kind, set_fullscreen, set_mouse_report,
    set_power_preference,
};
pub use present::feel::{
    og_camera_kick, og_camera_punch, og_camera_shake, og_load_flipbook, og_load_tilemap,
    og_render_floating_text, og_tilemap_cell_at, og_tilemap_cell_center, og_tilemap_create,
    og_tilemap_fill, og_tilemap_get_tile, og_tilemap_neighbours, og_tilemap_resize,
    og_tilemap_set_tile, og_time_group_dt, og_time_hit_stop, og_time_set_group_scale,
    og_time_set_scale, og_tween_active, og_tween_cancel, og_tween_cancel_entity, og_tween_flash,
    og_tween_from, og_tween_from_to, og_tween_parallel, og_tween_sequence, og_tween_squash,
    og_tween_timeline, og_tween_timeline_add, og_tween_timeline_add_label, og_tween_timeline_call,
    og_tween_timeline_duration, og_tween_timeline_from, og_tween_timeline_from_to,
    og_tween_timeline_get_error, og_tween_timeline_get_tween, og_tween_timeline_is_active,
    og_tween_timeline_kill, og_tween_timeline_pause, og_tween_timeline_play,
    og_tween_timeline_progress, og_tween_timeline_restart, og_tween_timeline_resume,
    og_tween_timeline_reverse, og_tween_timeline_seek, og_tween_timeline_set,
    og_tween_timeline_time, og_tween_timeline_time_scale, og_tween_timeline_to, og_tween_to,
};
pub use present::geometry::{
    og_debug_arrow, og_debug_axes, og_debug_box, og_debug_clear, og_debug_enabled, og_debug_line,
    og_debug_set_enabled, og_debug_set_view, og_debug_sphere, og_debug_text, og_debug_view,
    og_render_buffer_geometry, og_render_capsule_geometry, og_render_circle_geometry,
    og_render_cone_geometry, og_render_csg_geometry, og_render_cylinder_geometry,
    og_render_dodecahedron_geometry, og_render_extrude_geometry, og_render_geometry_info,
    og_render_icosahedron_geometry, og_render_lathe_geometry, og_render_lines,
    og_render_merge_geometries, og_render_octahedron_geometry, og_render_points,
    og_render_polyhedron_geometry, og_render_raycast, og_render_ring_geometry,
    og_render_shape_geometry, og_render_sweep_geometry, og_render_tetrahedron_geometry,
    og_render_torus_geometry, og_render_tube_geometry, og_render_update_geometry,
};
pub use present::light2d_at::og_render_light2d_at;
pub use present::loading::{
    og_loading_done, og_loading_idle, og_loading_stage, og_loading_status, og_render_compile,
    warm_tick,
};
pub use present::ui_draw::{og_load_ui_material, og_load_vector};
pub use present::ui_next::og_engine_ui_patch;
pub use present::views::{
    og_load_post, og_render_post_errors, og_render_render_target, og_render_set_post_param,
};
pub use world::assets::{Fetch, prepare_textures};
pub use world::save::{
    og_save_edit_close, og_save_edit_drop_type, og_save_edit_get_version, og_save_edit_records,
    og_save_edit_rename_field, og_save_edit_rename_type, og_save_edit_set_records,
    og_save_edit_spawn, og_save_edit_text, og_save_edit_types, og_world_open_save,
    og_world_restore, og_world_snapshot, save_snapshot_bytes,
};
pub use world::scenes::{
    og_scene_change, og_scene_current, og_scene_preload, og_scene_progress, og_scene_release,
};
pub use world::stream::{
    og_stream_budget, og_stream_release, og_stream_request, og_stream_set_budget, og_stream_status,
};
pub use world::uncooked::Decoded;

use serde_json::{Value, json};

/// `{ ok: true, value }` as wire text.
pub(crate) fn ok(value: Value) -> String {
    // The same text as `json!({ "ok": true, "value": value })`, without
    // the macro's copy of `value` (serde_json::to_value of a Value, a
    // sixth of og.world.get).
    format!(r#"{{"ok":true,"value":{value}}}"#)
}

/// `{ ok: true }`: a successful `Result<void>`.
pub(crate) fn ok_void() -> String {
    r#"{"ok":true}"#.to_string()
}

/// `{ ok: false, reason }` as wire text.
pub(crate) fn refuse(reason: impl Into<String>) -> String {
    json!({ "ok": false, "reason": reason.into() }).to_string()
}

/// Parses wire JSON; an absent optional argument is `null`.
pub(crate) fn parse(text: Option<&str>) -> Result<Value, String> {
    match text {
        None => Ok(Value::Null),
        Some(t) if t.is_empty() || t == "undefined" => Ok(Value::Null),
        Some(t) => serde_json::from_str(t).map_err(|e| format!("malformed JSON argument: {e}")),
    }
}

/// Installs the host's hooks (its dispatcher, view hand-out and clock).
pub fn set_hooks(hooks: std::rc::Rc<dyn Hooks>) {
    engine::HOOKS.with(|h| *h.borrow_mut() = Some(hooks));
}

/// The wall clock og-physics times its per-phase stats with
/// (`og.physics.stats`), callable on any thread the step runs on: natively
/// it has its own; the web host gives each Worker's `performance.now`.
pub fn set_physics_clock(clock: fn() -> f64) {
    og_physics::set_clock(clock);
}

/// The family TSX text in CSS family list `css` (`sans-serif`,
/// `"Cinzel", serif`) is laid out in first, as its font file names it:
/// the first of the list's families engine `engine`'s UI has a face for.
pub fn ui_text_family(engine: u32, css: &str) -> Option<String> {
    let eng = engine::engine(engine).ok()?;
    let mut rs = eng.render.borrow_mut();
    let c = &mut rs.ui_faces.collection;
    let (face, _) = c.primary(css, 400.0, false)?;
    c.family_name(face).map(str::to_owned)
}

/// The CSS family lists of engine `engine`'s text roles (sans, display,
/// mono, rounded), as og.ui.configure, its style pack and its locale left
/// them: what `font-sans`, `font-display`, `font-mono` and `font-rounded`
/// give a text.
pub fn ui_font_roles(engine: u32) -> [String; 4] {
    present::faces::roles(engine)
}

/// What a native window's AccessKit session was last sent ([`ui_a11y`]).
pub use og_ui_next::a11y as ui_a11y_client;

/// The new core's accessibility update for a native window's AccessKit
/// adapter, as AccessKit's JSON, when what the tree shows changed since
/// `client` was last updated (a new client: the whole tree). `width`: the
/// frame's physical pixels across.
pub fn ui_a11y(
    engine: u32,
    client: &mut og_ui_next::a11y::Tree,
    width: u32,
) -> Option<serde_json::Value> {
    let eng = engine::engine(engine).ok()?;
    let rs = eng.render.try_borrow().ok()?;
    let update = rs.ui_next.as_ref()?.a11y_native(client, width)?;
    Some(serde_json::to_value(update).expect("AccessKit's tree is JSON"))
}

/// A native window's screen reader asked an action of a node of the new
/// core's accessibility tree (AccessKit's `ActionRequest` as JSON): it
/// reaches the world as the web mirror's request does, an `a11y` input
/// naming the node by its id. Answers whether the world took it.
pub fn ui_a11y_action(engine: u32, request: serde_json::Value) -> bool {
    use og_ui_next::a11y::accesskit::{Action, ActionRequest};
    let Ok(request) = serde_json::from_value::<ActionRequest>(request) else {
        return false;
    };
    let action = match request.action {
        Action::Focus => og_input::A11yAct::Focus,
        Action::Click => og_input::A11yAct::Activate,
        Action::Increment => og_input::A11yAct::Increment,
        Action::Decrement => og_input::A11yAct::Decrement,
        _ => return false,
    };
    let Ok(eng) = engine::engine(engine) else {
        return false;
    };
    let Ok(world) = eng.world() else {
        return false;
    };
    let event = og_input::InjectedInput::A11y {
        action,
        path: request.target_node.0.to_string(),
    };
    og_input::InputSink::inject(&mut og_input::InputLayer::new(), world, &event)
}

/// The new core's TSX nodes as the probe lists them, and its focus order
/// as their paths: `og ui check` reads them (og-host-node `og_node_ui_frame`).
pub fn ui_next_dump(engine: u32) -> (Vec<og_probe::UiElement>, Vec<String>) {
    let Ok(eng) = engine::engine(engine) else {
        return Default::default();
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return Default::default();
    };
    rs.ui_next.as_ref().map(|n| n.op_dump()).unwrap_or_default()
}

/// The inspector's panel values since the last call (empty: unchanged;
/// `null`: closed), for the runtime's TSX panel.
pub fn og_engine_inspector_panel(engine: u32) -> String {
    #[cfg(not(feature = "publish"))]
    return author::inspector::take_panel(engine);
    #[cfg(feature = "publish")]
    {
        let _ = engine;
        String::new()
    }
}

/// One event of the inspector's TSX panel, done at the next frame.
pub fn og_engine_inspector_event(engine: u32, event: &str) -> bool {
    #[cfg(not(feature = "publish"))]
    return author::inspector::panel_event(engine, event);
    #[cfg(feature = "publish")]
    {
        let _ = (engine, event);
        false
    }
}

// The lifecycle members under their binding names, so a host's
// `Lifecycle` impl reads like its module impls.

pub fn og_engine_create(game: u32, options: String) -> String {
    create(game, &options)
}

pub fn og_engine_destroy(engine: u32) -> bool {
    destroy(engine)
}

pub fn og_engine_advance(engine: u32, steps: u32) -> String {
    advance(engine, steps)
}

pub fn og_engine_command(engine: u32, action: String, args: String) -> String {
    command(engine, &action, &args)
}

pub fn og_engine_frame(engine: u32, elapsed_ms: f64) -> String {
    frame(engine, elapsed_ms)
}

pub fn og_present_frame(engine: u32, now_ms: f64) -> String {
    present::pacing::frame(engine, now_ms)
}

pub fn og_present_resume(engine: u32) {
    present::pacing::resume(engine)
}

pub fn og_present_refresh(engine: u32, hz: f64) {
    present::pacing::refresh(engine, hz)
}

pub fn og_present_fullscreen(engine: u32, on: bool) {
    set_fullscreen(engine, on)
}

pub fn og_engine_write_resource(world: u32, resource: String, value: String) -> String {
    write_resource(world, &resource, &value)
}

pub fn og_engine_resource_layout(world: u32, resource: String) -> Result<Value, String> {
    engine::resource_layout(world, &resource)
}

pub fn og_engine_component_layout(world: u32, component: String) -> Result<Value, String> {
    engine::component_layout(world, &component)
}

pub fn og_engine_read_resource(world: u32, resource: String) -> Vec<u8> {
    engine::read_resource(world, &resource)
}

pub fn og_engine_write_resource_bytes(world: u32, resource: String, bytes: &[u8]) -> String {
    engine::write_resource_bytes(world, &resource, bytes)
}

pub fn og_engine_call(engine: u32, frame: String) -> String {
    call_rpc(engine, &frame)
}

pub fn og_engine_take_outputs(engine: u32) -> String {
    take_outputs(engine)
}

pub fn og_engine_take_log(engine: u32) -> String {
    take_log(engine)
}

pub fn og_host_begin_bundle() {
    begin_bundle()
}
