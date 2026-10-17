// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The napi exports of every binding symbol, under their snake_case
//! names: each calls the host's implementation of its `og_api::host` trait.

#![allow(clippy::too_many_arguments)]

use crate::NodeHost as Host;
use napi::Env;
use napi_derive::napi;
use og_api::host::*;

#[napi(js_name = "og_host_set_dispatcher")]
pub fn og_host_set_dispatcher(env: Env, dispatcher: crate::NodeFunction) {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_host_set_dispatcher(&Host, dispatcher)
}

#[napi(js_name = "og_engine_create")]
pub fn og_engine_create(env: Env, game: u32, options: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_create(&Host, game, options)
}

#[napi(js_name = "og_engine_destroy")]
pub fn og_engine_destroy(env: Env, engine: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_destroy(&Host, engine)
}

#[napi(js_name = "og_engine_advance")]
pub fn og_engine_advance(env: Env, engine: u32, steps: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_advance(&Host, engine, steps)
}

#[napi(js_name = "og_engine_command")]
pub fn og_engine_command(env: Env, engine: u32, action: String, args: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_command(&Host, engine, action, args)
}

#[napi(js_name = "og_engine_frame")]
pub fn og_engine_frame(env: Env, engine: u32, elapsed_ms: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_frame(&Host, engine, elapsed_ms)
}

#[napi(js_name = "og_engine_write_resource")]
pub fn og_engine_write_resource(env: Env, world: u32, resource: String, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_write_resource(&Host, world, resource, value)
}

#[napi(js_name = "og_reflect_define_component")]
pub fn og_reflect_define_component(env: Env, name: String, fields: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ReflectApi>::og_reflect_define_component(&Host, name, fields)
}

#[napi(js_name = "og_reflect_define_resource")]
pub fn og_reflect_define_resource(env: Env, name: String, fields: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ReflectApi>::og_reflect_define_resource(&Host, name, fields)
}

#[napi(js_name = "og_world_get_tick")]
pub fn og_world_get_tick(env: Env, world: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_get_tick(&Host, world)
}

#[napi(js_name = "og_world_spawn")]
pub fn og_world_spawn(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_spawn(&Host, world, desc)
}

#[napi(js_name = "og_world_despawn")]
pub fn og_world_despawn(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_despawn(&Host, world, entity)
}

#[napi(js_name = "og_world_alive")]
pub fn og_world_alive(env: Env, world: u32, entity: f64) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_alive(&Host, world, entity)
}

#[napi(js_name = "og_world_has")]
pub fn og_world_has(env: Env, world: u32, entity: f64, component: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_has(&Host, world, entity, component)
}

#[napi(js_name = "og_world_insert")]
pub fn og_world_insert(env: Env, world: u32, entity: f64, component: String, value: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_insert(&Host, world, entity, component, value)
}

#[napi(js_name = "og_world_remove")]
pub fn og_world_remove(env: Env, world: u32, entity: f64, component: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_remove(&Host, world, entity, component)
}

#[napi(js_name = "og_world_get")]
pub fn og_world_get(env: Env, world: u32, entity: f64, component: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_get(&Host, world, entity, component)
}

#[napi(js_name = "og_world_set")]
pub fn og_world_set(env: Env, world: u32, entity: f64, component: String, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_set(&Host, world, entity, component, value)
}

#[napi(js_name = "og_world_resource")]
pub fn og_world_resource(env: Env, world: u32, resource: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_resource(&Host, world, resource)
}

#[napi(js_name = "og_world_find")]
pub fn og_world_find(env: Env, world: u32, name: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_find(&Host, world, name)
}

#[napi(js_name = "og_world_name_of")]
pub fn og_world_name_of(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_name_of(&Host, world, entity)
}

#[napi(js_name = "og_world_query")]
pub fn og_world_query(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_query(&Host, world, desc)
}

#[napi(js_name = "og_world_events")]
pub fn og_world_events(env: Env, world: u32, event: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_events(&Host, world, event)
}

#[napi(js_name = "og_world_emit")]
pub fn og_world_emit(env: Env, world: u32, event: String, payload: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_emit(&Host, world, event, payload)
}

#[napi(js_name = "og_world_instantiate")]
pub fn og_world_instantiate(env: Env, world: u32, asset: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_world_instantiate(&Host, world, asset, desc)
}

#[napi(js_name = "og_query_archetypes")]
pub fn og_query_archetypes(env: Env, query: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_query_archetypes(&Host, query)
}

#[napi(js_name = "og_query_count")]
pub fn og_query_count(env: Env, query: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_query_count(&Host, query)
}

#[napi(js_name = "og_archetype_view_get_count")]
pub fn og_archetype_view_get_count(env: Env, archetype_view: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_get_count(&Host, archetype_view)
}

#[napi(js_name = "og_archetype_view_get_tick")]
pub fn og_archetype_view_get_tick(env: Env, archetype_view: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_get_tick(&Host, archetype_view)
}

#[napi(js_name = "og_archetype_view_get_entities")]
pub fn og_archetype_view_get_entities(env: Env, archetype_view: u32) -> crate::NodeArray {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_get_entities(&Host, archetype_view)
}

#[napi(js_name = "og_archetype_view_entity")]
pub fn og_archetype_view_entity(env: Env, archetype_view: u32, index: u32) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_entity(&Host, archetype_view, index)
}

#[napi(js_name = "og_archetype_view_has")]
pub fn og_archetype_view_has(env: Env, archetype_view: u32, component: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_has(&Host, archetype_view, component)
}

#[napi(js_name = "og_archetype_view_column")]
pub fn og_archetype_view_column(env: Env, archetype_view: u32, component: String) -> crate::NodeViews {
    let _env = crate::EnvScope::enter(&env);
    <Host as WorldApi>::og_archetype_view_column(&Host, archetype_view, component)
}

#[napi(js_name = "og_system_get_name")]
pub fn og_system_get_name(env: Env, system: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as SystemApi>::og_system_get_name(&Host, system)
}

#[napi(js_name = "og_system_get_reads")]
pub fn og_system_get_reads(env: Env, system: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as SystemApi>::og_system_get_reads(&Host, system)
}

#[napi(js_name = "og_system_get_writes")]
pub fn og_system_get_writes(env: Env, system: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as SystemApi>::og_system_get_writes(&Host, system)
}

#[napi(js_name = "og_system_define_system")]
pub fn og_system_define_system(env: Env, name: String, desc: String, run: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as SystemApi>::og_system_define_system(&Host, name, desc, run)
}

#[napi(js_name = "og_game_get_tick")]
pub fn og_game_get_tick(env: Env, game: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as GameApi>::og_game_get_tick(&Host, game)
}

#[napi(js_name = "og_game_get_systems")]
pub fn og_game_get_systems(env: Env, game: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as GameApi>::og_game_get_systems(&Host, game)
}

#[napi(js_name = "og_game_game")]
pub fn og_game_game(env: Env, desc: Option<String>) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as GameApi>::og_game_game(&Host, desc)
}

#[napi(js_name = "og_transform_compose")]
pub fn og_transform_compose(env: Env, position: String, quaternion: String, scale: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TransformApi>::og_transform_compose(&Host, position, quaternion, scale)
}

#[napi(js_name = "og_transform_decompose")]
pub fn og_transform_decompose(env: Env, matrix: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TransformApi>::og_transform_decompose(&Host, matrix)
}

#[napi(js_name = "og_transform_look_at")]
pub fn og_transform_look_at(env: Env, eye: String, target: String, up: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TransformApi>::og_transform_look_at(&Host, eye, target, up)
}

#[napi(js_name = "og_transform_from_axis_angle")]
pub fn og_transform_from_axis_angle(env: Env, axis: String, angle: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TransformApi>::og_transform_from_axis_angle(&Host, axis, angle)
}

#[napi(js_name = "og_rng_stream_get_seed")]
pub fn og_rng_stream_get_seed(env: Env, rng_stream: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_get_seed(&Host, rng_stream)
}

#[napi(js_name = "og_rng_stream_next")]
pub fn og_rng_stream_next(env: Env, rng_stream: u32) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_next(&Host, rng_stream)
}

#[napi(js_name = "og_rng_stream_range")]
pub fn og_rng_stream_range(env: Env, rng_stream: u32, low: f64, high: f64) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_range(&Host, rng_stream, low, high)
}

#[napi(js_name = "og_rng_stream_integer")]
pub fn og_rng_stream_integer(env: Env, rng_stream: u32, low: i32, high: i32) -> i32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_integer(&Host, rng_stream, low, high)
}

#[napi(js_name = "og_rng_stream_chance")]
pub fn og_rng_stream_chance(env: Env, rng_stream: u32, p: f64) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_chance(&Host, rng_stream, p)
}

#[napi(js_name = "og_rng_stream_fork")]
pub fn og_rng_stream_fork(env: Env, rng_stream: u32, label: String) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_stream_fork(&Host, rng_stream, label)
}

#[napi(js_name = "og_rng_next")]
pub fn og_rng_next(env: Env, world: u32) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_next(&Host, world)
}

#[napi(js_name = "og_rng_range")]
pub fn og_rng_range(env: Env, world: u32, low: f64, high: f64) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_range(&Host, world, low, high)
}

#[napi(js_name = "og_rng_integer")]
pub fn og_rng_integer(env: Env, world: u32, low: i32, high: i32) -> i32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_integer(&Host, world, low, high)
}

#[napi(js_name = "og_rng_chance")]
pub fn og_rng_chance(env: Env, world: u32, p: f64) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_chance(&Host, world, p)
}

#[napi(js_name = "og_rng_fork")]
pub fn og_rng_fork(env: Env, world: u32, label: String) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as RngApi>::og_rng_fork(&Host, world, label)
}

#[napi(js_name = "og_input_down")]
pub fn og_input_down(env: Env, world: u32, key: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_down(&Host, world, key)
}

#[napi(js_name = "og_input_pressed")]
pub fn og_input_pressed(env: Env, world: u32, key: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_pressed(&Host, world, key)
}

#[napi(js_name = "og_input_released")]
pub fn og_input_released(env: Env, world: u32, key: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_released(&Host, world, key)
}

#[napi(js_name = "og_input_axis")]
pub fn og_input_axis(env: Env, world: u32, negative: String, positive: String) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_axis(&Host, world, negative, positive)
}

#[napi(js_name = "og_input_button_down")]
pub fn og_input_button_down(env: Env, world: u32, button: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_button_down(&Host, world, button)
}

#[napi(js_name = "og_input_button_pressed")]
pub fn og_input_button_pressed(env: Env, world: u32, button: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_button_pressed(&Host, world, button)
}

#[napi(js_name = "og_camera_look_at")]
pub fn og_camera_look_at(env: Env, world: u32, target: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_look_at(&Host, world, target)
}

#[napi(js_name = "og_camera_world_to_screen")]
pub fn og_camera_world_to_screen(env: Env, world: u32, point: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_world_to_screen(&Host, world, point)
}

#[napi(js_name = "og_camera_screen_to_ray")]
pub fn og_camera_screen_to_ray(env: Env, world: u32, x: f64, y: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_screen_to_ray(&Host, world, x, y)
}

#[napi(js_name = "og_render_standard_material")]
pub fn og_render_standard_material(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_standard_material(&Host, world, desc)
}

#[napi(js_name = "og_render_update_material")]
pub fn og_render_update_material(env: Env, world: u32, material: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_update_material(&Host, world, material, desc)
}

#[napi(js_name = "og_render_capabilities")]
pub fn og_render_capabilities(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_capabilities(&Host, world)
}

#[napi(js_name = "og_load_gltf")]
pub fn og_load_gltf(env: Env, world: u32, path: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_gltf(&Host, world, path, desc)
}

#[napi(js_name = "og_load_scene")]
pub fn og_load_scene(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_scene(&Host, world, path)
}

#[napi(js_name = "og_load_status")]
pub fn og_load_status(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_status(&Host, world, asset)
}

#[napi(js_name = "og_load_gltf_info")]
pub fn og_load_gltf_info(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_gltf_info(&Host, world, asset)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_commands_register")]
pub fn og_commands_register(env: Env, name: String, handler: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CommandsApi>::og_commands_register(&Host, name, handler, desc)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_commands_parse")]
pub fn og_commands_parse(env: Env, script: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CommandsApi>::og_commands_parse(&Host, script)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_commands_schedule")]
pub fn og_commands_schedule(env: Env, world: u32, script: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CommandsApi>::og_commands_schedule(&Host, world, script)
}

#[napi(js_name = "og_runtime_probe_get_version")]
pub fn og_runtime_probe_get_version(env: Env, runtime_probe: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_get_version(&Host, runtime_probe)
}

#[napi(js_name = "og_runtime_probe_tick")]
pub fn og_runtime_probe_tick(env: Env, runtime_probe: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_tick(&Host, runtime_probe)
}

#[napi(js_name = "og_runtime_probe_state")]
pub fn og_runtime_probe_state(env: Env, runtime_probe: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_state(&Host, runtime_probe)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_advance")]
pub fn og_runtime_probe_advance(env: Env, runtime_probe: u32, steps: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_advance(&Host, runtime_probe, steps)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_pause")]
pub fn og_runtime_probe_pause(env: Env, runtime_probe: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_pause(&Host, runtime_probe)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_resume")]
pub fn og_runtime_probe_resume(env: Env, runtime_probe: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_resume(&Host, runtime_probe)
}

#[napi(js_name = "og_runtime_probe_input")]
pub fn og_runtime_probe_input(env: Env, runtime_probe: u32, event: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_input(&Host, runtime_probe, event)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_accepted")]
pub fn og_runtime_probe_accepted(env: Env, runtime_probe: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_accepted(&Host, runtime_probe)
}

#[napi(js_name = "og_runtime_probe_pick")]
pub fn og_runtime_probe_pick(env: Env, runtime_probe: u32, x: f64, y: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_pick(&Host, runtime_probe, x, y)
}

#[napi(js_name = "og_runtime_probe_entity")]
pub fn og_runtime_probe_entity(env: Env, runtime_probe: u32, id: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_entity(&Host, runtime_probe, id)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_bounds")]
pub fn og_runtime_probe_bounds(env: Env, runtime_probe: u32, id: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_bounds(&Host, runtime_probe, id)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_patch")]
pub fn og_runtime_probe_patch(env: Env, runtime_probe: u32, id: String, parameter: String, value: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_patch(&Host, runtime_probe, id, parameter, value)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_runtime_probe_run")]
pub fn og_runtime_probe_run(env: Env, runtime_probe: u32, program: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_runtime_probe_run(&Host, runtime_probe, program)
}

#[napi(js_name = "og_engine_probe_hello")]
pub fn og_engine_probe_hello(env: Env, engine_probe: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_hello(&Host, engine_probe)
}

#[napi(js_name = "og_engine_probe_capture")]
pub fn og_engine_probe_capture(env: Env, engine_probe: u32, width: Option<u32>, height: Option<u32>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_capture(&Host, engine_probe, width, height)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_sweep")]
pub fn og_engine_probe_sweep(env: Env, engine_probe: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_sweep(&Host, engine_probe, desc)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_reload")]
pub fn og_engine_probe_reload(env: Env, engine_probe: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_reload(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_snapshot")]
pub fn og_engine_probe_snapshot(env: Env, engine_probe: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_snapshot(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_restore")]
pub fn og_engine_probe_restore(env: Env, engine_probe: u32, id: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_restore(&Host, engine_probe, id)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_dump")]
pub fn og_engine_probe_dump(env: Env, engine_probe: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_dump(&Host, engine_probe, desc)
}

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_commands")]
pub fn og_engine_probe_commands(env: Env, engine_probe: u32, script: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_commands(&Host, engine_probe, script)
}
