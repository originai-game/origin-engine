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

mod ai;
mod anim;
mod api;
mod assets;
mod codec;
mod e6;
mod engine;
mod extract;
mod feel;
mod fx;
mod i18n;
mod modules;
mod nav;
mod platform;
mod polish;
mod probe;

pub use ai::{og_ai_describe, og_ai_get, og_ai_load, og_ai_noise, og_ai_parse, og_ai_set};
pub use anim::{og_anim_graph, og_load_animation};
pub use api::*;
pub use assets::Fetch;
pub use codec::{Kind, Layout, RecordKind};
pub use engine::{
    ColumnPtr, Dispatch, Hooks, RenderBackend, advance, answer_call, begin_bundle, call_rpc,
    command, create, destroy, frame, render_options, set_render_backend, take_log, take_outputs,
    view_columns, view_entities, write_resource,
};
pub use feel::{
    og_camera_kick, og_camera_punch, og_camera_shake, og_load_flipbook, og_load_tilemap,
    og_render_floating_text, og_tilemap_cell_at, og_tilemap_cell_center, og_tilemap_get_tile,
    og_tilemap_set_tile, og_time_hit_stop, og_time_set_scale, og_tween_active, og_tween_cancel,
    og_tween_cancel_entity, og_tween_flash, og_tween_parallel, og_tween_sequence, og_tween_squash,
    og_tween_to,
};
pub use i18n::{og_i18n_has, og_i18n_load, og_i18n_locale, og_i18n_set_locale, og_i18n_t};
pub use modules::{
    THEME_COLORS, TYPE_STEPS, og_load_skin, og_physics_overlap, og_physics_raycast,
    og_physics_set_mesh, og_physics_shape_cast, og_ui_action_button, og_ui_back, og_ui_badge,
    og_ui_bounds, og_ui_breakpoint, og_ui_burst, og_ui_button, og_ui_checkbox, og_ui_checkbox_list,
    og_ui_chip, og_ui_close, og_ui_close_list, og_ui_close_modal, og_ui_configure, og_ui_cooldown,
    og_ui_counter, og_ui_dialogue, og_ui_dismiss_toast, og_ui_divider, og_ui_dpad, og_ui_dragging,
    og_ui_frame_theme, og_ui_frame_viewport, og_ui_grid_item, og_ui_has_icon, og_ui_icon,
    og_ui_image, og_ui_joystick, og_ui_key_value, og_ui_label, og_ui_list_item, og_ui_memory,
    og_ui_meter, og_ui_minimap, og_ui_navigate, og_ui_open, og_ui_open_anchor, og_ui_open_grid,
    og_ui_open_list, og_ui_open_modal, og_ui_open_panel, og_ui_open_scroll, og_ui_open_transition,
    og_ui_preview_viewport, og_ui_progress, og_ui_radio, og_ui_remember, og_ui_screen_transition,
    og_ui_seconds, og_ui_select, og_ui_set_navigation, og_ui_skeleton, og_ui_slider, og_ui_spacer,
    og_ui_spinner, og_ui_spotlight, og_ui_stepper, og_ui_tabs, og_ui_text_field, og_ui_theme,
    og_ui_tick, og_ui_toast, og_ui_toggle, og_ui_tooltip, og_ui_viewport, set_viewport,
};
pub use nav::{
    og_nav_build, og_nav_cell_at, og_nav_find_path, og_nav_flow_at, og_nav_flow_field, og_nav_grid,
    og_nav_grid_from_tilemap, og_nav_load, og_nav_nearest, og_nav_path_of, og_nav_set_cost,
    og_nav_set_flow_targets, og_nav_set_obstacle,
};
pub use platform::{
    og_platform_ai_mark, og_platform_answer, og_platform_info, og_platform_leaderboard_around,
    og_platform_leaderboard_submit, og_platform_leaderboard_top, og_platform_load_settings,
    og_platform_point, og_platform_result, og_platform_save_settings, og_platform_set_host,
    og_platform_store_get, og_platform_store_list, og_platform_store_set, og_platform_take,
    og_platform_telemetry,
};
pub use probe::png;

use serde_json::{Value, json};

/// `{ ok: true, value }` as wire text.
pub(crate) fn ok(value: Value) -> String {
    json!({ "ok": true, "value": value }).to_string()
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
