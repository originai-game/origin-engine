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
//! Single-threaded by construction: every table is thread-local, and an
//! engine never leaves the thread that created it.

#![recursion_limit = "256"]

#[cfg(feature = "ai")]
mod ai;
// Without the `ai` feature og-ai is not linked: og.ai answers the refusal
// (src/ai_absent.rs).
#[cfg(not(feature = "ai"))]
#[path = "ai_absent.rs"]
mod ai;
mod anim;
mod api;
mod art;
mod assets;
mod atmosphere;
mod baked;
mod codec;
mod describe;
#[cfg(feature = "destruction")]
mod destruction;
#[cfg(feature = "destruction")]
mod destruction2d;
// Without the `destruction` feature og-destruct is not linked: both halves
// of og.destruction answer the refusal (src/destruction*_absent.rs).
#[cfg(not(feature = "destruction"))]
#[path = "destruction_absent.rs"]
mod destruction;
#[cfg(not(feature = "destruction"))]
#[path = "destruction2d_absent.rs"]
mod destruction2d;
#[cfg(not(feature = "publish"))]
mod determinism;
mod display;
mod e6;
mod e6_idle;
mod effect;
mod engine;
mod environment;
mod extract;
mod feel;
mod fx;
mod garment;
mod geometry;
mod glyphs;
pub mod host_impl;
pub use glyphs::glyphs_pending;
mod i18n;
#[cfg(not(feature = "publish"))]
mod inspector;
mod interpolate;
mod invariants;
mod lanes;
mod lazy;
mod light2d_at;
mod look;
mod memory;
mod probes;
mod skeleton2d;
pub use lanes::{panic_report, record_panic, report as lanes_report};
mod modules;
#[cfg(feature = "motion")]
mod motion;
// Without the `motion` feature og-motion is not linked: og.motion answers
// the refusal (src/motion_absent.rs).
#[cfg(not(feature = "motion"))]
#[path = "motion_absent.rs"]
mod motion;
#[cfg(feature = "nav")]
mod nav;
// Without the `nav` feature og-nav is not linked: og.nav answers the
// refusal (src/nav_absent.rs).
#[cfg(not(feature = "nav"))]
#[path = "nav_absent.rs"]
mod nav;
mod net;
mod origin;
mod particle_events;
mod physics_ext;
mod platform;
mod prefab;
mod presentation;
mod probe;
mod procgen;
mod pulse;
#[cfg(not(feature = "publish"))]
mod record;
mod review;
mod save;
mod save_migrations;
#[cfg(not(feature = "publish"))]
mod scene_save;
mod seabed;
pub mod shared;
#[cfg(feature = "story")]
mod story;
// Without the `story` feature og-story is not linked: og.story answers the
// refusal (src/story_absent.rs).
#[cfg(not(feature = "story"))]
#[path = "story_absent.rs"]
mod story;
mod stream;
#[cfg(not(feature = "profile2d"))]
mod terrain;
// The 2D profile leaves the 3D world out: its terrain answers the refusal.
#[cfg(feature = "profile2d")]
#[path = "terrain_absent.rs"]
mod terrain;
#[cfg(not(feature = "profile2d"))]
mod terrain_stream;
mod timeline;
mod trace;
pub mod ui_cursor;
mod ui_draw;
mod uncooked;
mod vfx;
#[cfg(feature = "video")]
mod video;
// Without the `video` feature og-video is not linked: og.video answers the
// refusal (src/video_absent.rs).
#[cfg(not(feature = "video"))]
#[path = "video_absent.rs"]
mod video;
mod views;
mod weather;
pub mod wire;

pub use ai::{
    og_ai_describe, og_ai_finish, og_ai_get, og_ai_load, og_ai_noise, og_ai_parse, og_ai_restart,
    og_ai_set,
};
pub use anim::{
    og_anim_graph, og_anim_humanoid, og_anim_mirror, og_anim_retarget, og_load_animation,
    og_load_rig,
};
pub use api::*;
pub use assets::Fetch;
pub use atmosphere::{
    og_atmosphere_time_of_day, og_load_shader, og_render_set_material_shader,
    og_render_set_shader_param, og_render_shader_material,
};
pub use baked::og_render_baked_lighting;
pub use codec::{Kind, Layout, RecordKind};
pub use destruction::{
    og_destruction_capabilities, og_destruction_damage, og_destruction_explode,
    og_destruction_fracture, og_destruction_info, og_destruction_material,
};
pub use destruction2d::{
    og_destruction_carve2d, og_destruction_cell2d, og_destruction_explode2d, og_destruction_fill2d,
    og_destruction_fracture2d,
};
pub use display::{
    lose_device, og_render_display, og_render_reset_display, og_render_set_display,
    set_capture_only, set_display_interval, set_fullscreen, set_mouse_report, set_power_preference,
    set_scripted_frames,
};
pub use engine::{
    ColumnPtr, Dispatch, Hooks, RenderBackend, VideoDecoder, advance, answer_call, antialias_name,
    begin_bundle, call_rpc, command, create, destroy, frame, point_shadows_name, render_desc,
    render_options, set_render_backend, take_log, take_outputs, transparency_name, view_columns,
    view_entities, warm_renderer, write_resource,
};
pub use feel::{
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
pub use geometry::{
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
pub use i18n::{
    fonts_pending, og_i18n_format_compact, og_i18n_format_currency, og_i18n_format_date,
    og_i18n_format_duration, og_i18n_format_number, og_i18n_format_time, og_i18n_has, og_i18n_load,
    og_i18n_locale, og_i18n_set_locale, og_i18n_t,
};
pub use invariants::og_assert_record;
pub use light2d_at::og_render_light2d_at;
pub use memory::memory_report;
pub use modules::ui_world::{og_ui_define_mass, og_ui_mass_count, og_ui_spawn_mass};
pub use modules::{
    THEME_COLORS, TYPE_STEPS, og_load_document, og_load_style, og_physics_overlap,
    og_physics_project_point, og_physics_raycast, og_physics_raycast_all, og_physics_set_mesh,
    og_physics_shape_cast, og_ui_a11y, og_ui_action, og_ui_action_bar, og_ui_action_held,
    og_ui_animate, og_ui_back, og_ui_bounds, og_ui_clear, og_ui_configure, og_ui_define_action,
    og_ui_define_timeline, og_ui_device, og_ui_dismiss_toast, og_ui_doc_events,
    og_ui_document_schema, og_ui_feedback, og_ui_file_picker, og_ui_focus, og_ui_force_state,
    og_ui_frame_cursor, og_ui_frame_direction, og_ui_frame_glyph, og_ui_frame_pointer,
    og_ui_frame_theme, og_ui_frame_viewport, og_ui_game_input, og_ui_gesture, og_ui_glyph,
    og_ui_has_icon, og_ui_hud_edit, og_ui_hud_layout, og_ui_hud_reset, og_ui_hud_select,
    og_ui_hud_step, og_ui_layers, og_ui_layout_stats, og_ui_motion_events, og_ui_navigate,
    og_ui_overlaps, og_ui_pause, og_ui_paused, og_ui_play, og_ui_play_info, og_ui_play_timeline,
    og_ui_pop, og_ui_preview_viewport, og_ui_push, og_ui_replace, og_ui_resume, og_ui_reverse,
    og_ui_save_edits, og_ui_screen_vm, og_ui_scroll_into_view, og_ui_scroll_state, og_ui_seconds,
    og_ui_seek, og_ui_set_device, og_ui_set_focus, og_ui_set_haptics, og_ui_set_hud_edit,
    og_ui_set_hud_layout, og_ui_set_hud_player, og_ui_set_navigation, og_ui_set_players,
    og_ui_set_prop, og_ui_set_virtual_cursor, og_ui_stop, og_ui_theme, og_ui_tick, og_ui_toast,
    og_ui_viewport, og_ui_virtual_cursor, og_ui_visual_state, set_viewport,
};
pub use motion::{
    og_motion_capabilities, og_motion_clear, og_motion_face, og_motion_jump, og_motion_look_at,
    og_motion_place, og_motion_pose, og_motion_poses, og_motion_presentation, og_motion_react,
    og_motion_set_backend, og_motion_status, og_motion_walk_to,
};
pub use nav::{
    og_nav_build, og_nav_carve, og_nav_cell_at, og_nav_find_path, og_nav_finish_link,
    og_nav_flow_at, og_nav_flow_field, og_nav_grid, og_nav_grid_from_tilemap, og_nav_load,
    og_nav_nearest, og_nav_path_of, og_nav_set_area, og_nav_set_cost, og_nav_set_flow_targets,
    og_nav_set_link, og_nav_set_link_open, og_nav_set_obstacle,
};
pub use net::{
    og_net_command, og_net_commands, og_net_final_hash, og_net_hash, og_net_resimulating,
    og_net_shape, og_net_status, og_net_submit, og_net_submit_input, og_platform_quick_play,
    og_platform_room_create, og_platform_room_info, og_platform_room_join, og_platform_room_leave,
    og_platform_room_message, og_platform_room_peers, og_platform_room_send, og_room_deliver,
    og_room_set_host, og_room_take, world_snapshot_bytes,
};
/// The counting allocator the hosts install (`#[global_allocator]`), for the probe's `memory`.
pub use og_core::alloc::Counting as CountingAlloc;
pub use og_render::png;
pub use physics_ext::{
    og_physics_apply_force, og_physics_apply_impulse, og_physics_apply_torque,
    og_physics_apply_torque_impulse, og_physics_cloth_points, og_physics_fluid_particles,
    og_physics_fluid3d_particles, og_physics_garment_points, og_physics_ragdoll_pose,
    og_physics_ragdoll_rig_from_skeleton, og_physics_rope_points, og_physics_set_ragdoll_rig,
    og_physics_soft_body_points, og_physics_stats, og_physics_vehicle_wheels,
    og_physics_water_height,
};
pub use platform::{
    og_platform_achievement_progress, og_platform_achievement_unlock, og_platform_achievements,
    og_platform_ad_ready, og_platform_ai_mark, og_platform_answer, og_platform_consume,
    og_platform_entitlements, og_platform_friends, og_platform_grant, og_platform_granted,
    og_platform_info, og_platform_invite_send, og_platform_invites, og_platform_leaderboard_around,
    og_platform_leaderboard_submit, og_platform_leaderboard_top, og_platform_load_settings,
    og_platform_load_slot, og_platform_notice, og_platform_notify, og_platform_now,
    og_platform_player, og_platform_point, og_platform_products, og_platform_purchase,
    og_platform_result, og_platform_save_settings, og_platform_save_slot, og_platform_set_host,
    og_platform_share, og_platform_show_ad, og_platform_store_get, og_platform_store_list,
    og_platform_store_set, og_platform_supports, og_platform_take, og_platform_telemetry,
};
pub use procgen::{
    og_procgen_bsp, og_procgen_noise, og_procgen_noise_grid, og_procgen_poisson_disk,
    og_procgen_wfc,
};
#[cfg(not(feature = "publish"))]
pub use record::{render_frame, render_restore, render_snapshot};
pub use save::{
    og_save_edit_close, og_save_edit_drop_type, og_save_edit_get_version, og_save_edit_records,
    og_save_edit_rename_field, og_save_edit_rename_type, og_save_edit_set_records,
    og_save_edit_spawn, og_save_edit_text, og_save_edit_types, og_world_open_save,
    og_world_restore, og_world_snapshot, save_snapshot_bytes,
};
pub use skeleton2d::{og_anim_rig2d_info, og_anim_skeleton2d_boxes, og_load_rig2d};
pub use story::{
    og_story_advance, og_story_ambient, og_story_bark, og_story_barks, og_story_choose,
    og_story_complete_quest, og_story_conversations, og_story_fail_quest, og_story_get,
    og_story_history, og_story_load, og_story_mark, og_story_mark_seen, og_story_parse,
    og_story_progress, og_story_quest, og_story_quests, og_story_reset, og_story_set,
    og_story_set_saliency, og_story_set_stage, og_story_start, og_story_start_quest, og_story_stop,
    og_story_track, og_story_view,
};
pub use stream::{
    og_stream_budget, og_stream_release, og_stream_request, og_stream_set_budget, og_stream_status,
};
pub use terrain::{
    og_terrain_bounds, og_terrain_edit, og_terrain_layer, og_terrain_load, og_terrain_sample,
    og_terrain_set_layer, og_terrain_stamp, og_terrain_unstamp,
};
pub use timeline::{
    og_load_timeline, og_path_create, og_path_find, og_path_length, og_path_nearest,
    og_path_sample, og_timeline_info, og_timeline_pause, og_timeline_play, og_timeline_seek,
    og_timeline_stop,
};
pub use trace::{BEFORE_ENGINE as TRACE_BEFORE_ENGINE, set as trace_set, take as trace_take};
pub use ui_draw::{og_load_ui_material, og_load_vector};
pub use uncooked::Decoded;
pub use vfx::{og_load_vfx, og_vfx_info, og_vfx_spawn};
pub use video::{og_load_video, og_load_video_info, og_video_capabilities};
pub use views::{
    og_load_post, og_render_post_errors, og_render_render_target, og_render_set_post_param,
};

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

/// An engine's UI between ticks: its last frame, theme and viewport, which
/// the native host's `og ui check` reads (og-host-node's `og_node_ui_frame`,
/// docs/ui.md §6). `None` for an engine without one.
pub fn shared_ui(engine: u32) -> Option<og_ui::SharedUi> {
    engine::engine(engine).ok()?.state.borrow().ui.clone()
}

/// The binary, host-neutral retained UI protocol.
pub fn og_engine_ui_patch(world: u32, bytes: &[u8]) -> Vec<u8> {
    match shared_ui(world) {
        Some(ui) => ui.lock().apply_reactive(bytes),
        None => og_ui::reactive::wire::encode(&serde_json::json!({
            "ok": false, "reason": "og.ui: world has no UI"
        }))
        .expect("bounded error reply"),
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

pub fn og_engine_write_resource(world: u32, resource: String, value: String) -> String {
    write_resource(world, &resource, &value)
}

pub fn og_engine_resource_layout(world: u32, resource: String) -> String {
    engine::resource_layout(world, &resource)
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
