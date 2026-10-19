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

#[napi(js_name = "og_host_begin_bundle")]
pub fn og_host_begin_bundle(env: Env) {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_host_begin_bundle(&Host)
}

#[napi(js_name = "og_engine_call")]
pub fn og_engine_call(env: Env, engine: u32, frame: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_call(&Host, engine, frame)
}

#[napi(js_name = "og_engine_take_outputs")]
pub fn og_engine_take_outputs(env: Env, engine: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_take_outputs(&Host, engine)
}

#[napi(js_name = "og_engine_take_log")]
pub fn og_engine_take_log(env: Env, engine: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_take_log(&Host, engine)
}

#[napi(js_name = "og_engine_write_resource")]
pub fn og_engine_write_resource(env: Env, world: u32, resource: String, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_engine_write_resource(&Host, world, resource, value)
}

#[napi(js_name = "og_platform_take")]
pub fn og_platform_take(env: Env, engine: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_platform_take(&Host, engine)
}

#[napi(js_name = "og_platform_answer")]
pub fn og_platform_answer(env: Env, engine: u32, request: u32, answer: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_platform_answer(&Host, engine, request, answer)
}

#[napi(js_name = "og_platform_set_host")]
pub fn og_platform_set_host(env: Env, engine: u32, info: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as Lifecycle>::og_platform_set_host(&Host, engine, info)
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

#[napi(js_name = "og_time_set_scale")]
pub fn og_time_set_scale(env: Env, world: u32, scale: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TimeApi>::og_time_set_scale(&Host, world, scale)
}

#[napi(js_name = "og_time_hit_stop")]
pub fn og_time_hit_stop(env: Env, world: u32, ticks: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TimeApi>::og_time_hit_stop(&Host, world, ticks)
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

#[napi(js_name = "og_input_axis2")]
pub fn og_input_axis2(env: Env, world: u32, left: String, right: String, down: String, up: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_axis2(&Host, world, left, right, down, up)
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

#[napi(js_name = "og_input_button_released")]
pub fn og_input_button_released(env: Env, world: u32, button: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_input_button_released(&Host, world, button)
}

#[napi(js_name = "og_pointers_down")]
pub fn og_pointers_down(env: Env, world: u32, id: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_down(&Host, world, id)
}

#[napi(js_name = "og_pointers_pressed")]
pub fn og_pointers_pressed(env: Env, world: u32, id: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_pressed(&Host, world, id)
}

#[napi(js_name = "og_pointers_released")]
pub fn og_pointers_released(env: Env, world: u32, id: u32) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_released(&Host, world, id)
}

#[napi(js_name = "og_pointers_position")]
pub fn og_pointers_position(env: Env, world: u32, id: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_position(&Host, world, id)
}

#[napi(js_name = "og_pointers_start")]
pub fn og_pointers_start(env: Env, world: u32, id: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_start(&Host, world, id)
}

#[napi(js_name = "og_pointers_count")]
pub fn og_pointers_count(env: Env, world: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_pointers_count(&Host, world)
}

#[napi(js_name = "og_gamepad_connected")]
pub fn og_gamepad_connected(env: Env, world: u32, pad: Option<u32>) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_gamepad_connected(&Host, world, pad)
}

#[napi(js_name = "og_gamepad_down")]
pub fn og_gamepad_down(env: Env, world: u32, button: String, pad: Option<u32>) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_gamepad_down(&Host, world, button, pad)
}

#[napi(js_name = "og_gamepad_pressed")]
pub fn og_gamepad_pressed(env: Env, world: u32, button: String, pad: Option<u32>) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_gamepad_pressed(&Host, world, button, pad)
}

#[napi(js_name = "og_gamepad_released")]
pub fn og_gamepad_released(env: Env, world: u32, button: String, pad: Option<u32>) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_gamepad_released(&Host, world, button, pad)
}

#[napi(js_name = "og_gamepad_axis")]
pub fn og_gamepad_axis(env: Env, world: u32, axis: String, pad: Option<u32>) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as InputApi>::og_gamepad_axis(&Host, world, axis, pad)
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

#[napi(js_name = "og_camera_shake")]
pub fn og_camera_shake(env: Env, world: u32, trauma: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_shake(&Host, world, trauma)
}

#[napi(js_name = "og_camera_punch")]
pub fn og_camera_punch(env: Env, world: u32, amount: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_punch(&Host, world, amount)
}

#[napi(js_name = "og_camera_kick")]
pub fn og_camera_kick(env: Env, world: u32, direction: String, distance: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CameraApi>::og_camera_kick(&Host, world, direction, distance)
}

#[napi(js_name = "og_render_ribbon")]
pub fn og_render_ribbon(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_ribbon(&Host, world, desc)
}

#[napi(js_name = "og_render_sprite")]
pub fn og_render_sprite(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_sprite(&Host, world, desc)
}

#[napi(js_name = "og_render_floating_text")]
pub fn og_render_floating_text(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_floating_text(&Host, world, desc)
}

#[napi(js_name = "og_render_measure_text")]
pub fn og_render_measure_text(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_measure_text(&Host, world, desc)
}

#[napi(js_name = "og_render_box_geometry")]
pub fn og_render_box_geometry(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_box_geometry(&Host, world, desc)
}

#[napi(js_name = "og_render_plane_geometry")]
pub fn og_render_plane_geometry(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_plane_geometry(&Host, world, desc)
}

#[napi(js_name = "og_render_sphere_geometry")]
pub fn og_render_sphere_geometry(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_sphere_geometry(&Host, world, desc)
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

#[napi(js_name = "og_render_shader_material")]
pub fn og_render_shader_material(env: Env, world: u32, shader: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_shader_material(&Host, world, shader, desc)
}

#[napi(js_name = "og_render_set_material_shader")]
pub fn og_render_set_material_shader(env: Env, world: u32, material: f64, shader: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_set_material_shader(&Host, world, material, shader)
}

#[napi(js_name = "og_render_set_shader_param")]
pub fn og_render_set_shader_param(env: Env, world: u32, material: f64, name: String, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_set_shader_param(&Host, world, material, name, value)
}

#[napi(js_name = "og_render_capabilities")]
pub fn og_render_capabilities(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_capabilities(&Host, world)
}

#[napi(js_name = "og_render_text")]
pub fn og_render_text(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as RenderApi>::og_render_text(&Host, world, desc)
}

#[napi(js_name = "og_load_bundle")]
pub fn og_load_bundle(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_bundle(&Host, world, path)
}

#[napi(js_name = "og_load_bundle_info")]
pub fn og_load_bundle_info(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_bundle_info(&Host, world, asset)
}

#[napi(js_name = "og_load_default_bundle")]
pub fn og_load_default_bundle(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_default_bundle(&Host, world)
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

#[napi(js_name = "og_load_font")]
pub fn og_load_font(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_font(&Host, world, path)
}

#[napi(js_name = "og_load_font_info")]
pub fn og_load_font_info(env: Env, world: u32, font: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_font_info(&Host, world, font)
}

#[napi(js_name = "og_load_sound")]
pub fn og_load_sound(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_sound(&Host, world, path)
}

#[napi(js_name = "og_load_lut")]
pub fn og_load_lut(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_lut(&Host, world, path)
}

#[napi(js_name = "og_load_emitter")]
pub fn og_load_emitter(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_emitter(&Host, world, path)
}

#[napi(js_name = "og_load_shader")]
pub fn og_load_shader(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_shader(&Host, world, path)
}

#[napi(js_name = "og_load_texture")]
pub fn og_load_texture(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_texture(&Host, world, path)
}

#[napi(js_name = "og_load_skin")]
pub fn og_load_skin(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_skin(&Host, world, path)
}

#[napi(js_name = "og_load_sound_info")]
pub fn og_load_sound_info(env: Env, world: u32, sound: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_sound_info(&Host, world, sound)
}

#[napi(js_name = "og_load_status")]
pub fn og_load_status(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_status(&Host, world, asset)
}

#[napi(js_name = "og_load_error")]
pub fn og_load_error(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_error(&Host, world, asset)
}

#[napi(js_name = "og_load_gltf_info")]
pub fn og_load_gltf_info(env: Env, world: u32, asset: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_gltf_info(&Host, world, asset)
}

#[napi(js_name = "og_load_animation")]
pub fn og_load_animation(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_animation(&Host, world, path)
}

#[napi(js_name = "og_load_rig")]
pub fn og_load_rig(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_rig(&Host, world, path)
}

#[napi(js_name = "og_load_flipbook")]
pub fn og_load_flipbook(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_flipbook(&Host, world, path)
}

#[napi(js_name = "og_load_tilemap")]
pub fn og_load_tilemap(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as LoadApi>::og_load_tilemap(&Host, world, path)
}

#[napi(js_name = "og_physics_raycast")]
pub fn og_physics_raycast(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_raycast(&Host, world, desc)
}

#[napi(js_name = "og_physics_shape_cast")]
pub fn og_physics_shape_cast(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_shape_cast(&Host, world, desc)
}

#[napi(js_name = "og_physics_overlap")]
pub fn og_physics_overlap(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_overlap(&Host, world, desc)
}

#[napi(js_name = "og_physics_set_mesh")]
pub fn og_physics_set_mesh(env: Env, world: u32, mesh: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_set_mesh(&Host, world, mesh, desc)
}

#[napi(js_name = "og_physics_rope_points")]
pub fn og_physics_rope_points(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_rope_points(&Host, world, entity)
}

#[napi(js_name = "og_physics_cloth_points")]
pub fn og_physics_cloth_points(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_cloth_points(&Host, world, entity)
}

#[napi(js_name = "og_physics_fluid_particles")]
pub fn og_physics_fluid_particles(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_fluid_particles(&Host, world, entity)
}

#[napi(js_name = "og_physics_water_height")]
pub fn og_physics_water_height(env: Env, world: u32, entity: f64, x: f64, z: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_water_height(&Host, world, entity, x, z)
}

#[napi(js_name = "og_physics_vehicle_wheels")]
pub fn og_physics_vehicle_wheels(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_vehicle_wheels(&Host, world, entity)
}

#[napi(js_name = "og_physics_ragdoll_rig_from_skeleton")]
pub fn og_physics_ragdoll_rig_from_skeleton(env: Env, world: u32, skeleton: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_ragdoll_rig_from_skeleton(&Host, world, skeleton)
}

#[napi(js_name = "og_physics_set_ragdoll_rig")]
pub fn og_physics_set_ragdoll_rig(env: Env, world: u32, entity: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_set_ragdoll_rig(&Host, world, entity, desc)
}

#[napi(js_name = "og_physics_ragdoll_pose")]
pub fn og_physics_ragdoll_pose(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PhysicsApi>::og_physics_ragdoll_pose(&Host, world, entity)
}

#[napi(js_name = "og_atmosphere_time_of_day")]
pub fn og_atmosphere_time_of_day(env: Env, world: u32) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as AtmosphereApi>::og_atmosphere_time_of_day(&Host, world)
}

#[napi(js_name = "og_anim_graph")]
pub fn og_anim_graph(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AnimApi>::og_anim_graph(&Host, world, desc)
}

#[napi(js_name = "og_anim_humanoid")]
pub fn og_anim_humanoid(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AnimApi>::og_anim_humanoid(&Host, world, desc)
}

#[napi(js_name = "og_anim_retarget")]
pub fn og_anim_retarget(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AnimApi>::og_anim_retarget(&Host, world, desc)
}

#[napi(js_name = "og_tween_to")]
pub fn og_tween_to(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_to(&Host, world, desc)
}

#[napi(js_name = "og_tween_sequence")]
pub fn og_tween_sequence(env: Env, world: u32, steps: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_sequence(&Host, world, steps, desc)
}

#[napi(js_name = "og_tween_parallel")]
pub fn og_tween_parallel(env: Env, world: u32, tweens: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_parallel(&Host, world, tweens, desc)
}

#[napi(js_name = "og_tween_cancel")]
pub fn og_tween_cancel(env: Env, world: u32, tween: f64) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_cancel(&Host, world, tween)
}

#[napi(js_name = "og_tween_cancel_entity")]
pub fn og_tween_cancel_entity(env: Env, world: u32, entity: f64) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_cancel_entity(&Host, world, entity)
}

#[napi(js_name = "og_tween_active")]
pub fn og_tween_active(env: Env, world: u32, tween: f64) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_active(&Host, world, tween)
}

#[napi(js_name = "og_tween_squash")]
pub fn og_tween_squash(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_squash(&Host, world, desc)
}

#[napi(js_name = "og_tween_flash")]
pub fn og_tween_flash(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TweenApi>::og_tween_flash(&Host, world, desc)
}

#[napi(js_name = "og_tilemap_set_tile")]
pub fn og_tilemap_set_tile(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TilemapApi>::og_tilemap_set_tile(&Host, world, desc)
}

#[napi(js_name = "og_tilemap_get_tile")]
pub fn og_tilemap_get_tile(env: Env, world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as TilemapApi>::og_tilemap_get_tile(&Host, world, entity, layer, x, y)
}

#[napi(js_name = "og_tilemap_cell_at")]
pub fn og_tilemap_cell_at(env: Env, world: u32, entity: f64, point: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TilemapApi>::og_tilemap_cell_at(&Host, world, entity, point)
}

#[napi(js_name = "og_tilemap_cell_center")]
pub fn og_tilemap_cell_center(env: Env, world: u32, entity: f64, x: u32, y: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TilemapApi>::og_tilemap_cell_center(&Host, world, entity, x, y)
}

#[napi(js_name = "og_nav_grid")]
pub fn og_nav_grid(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_grid(&Host, world, desc)
}

#[napi(js_name = "og_nav_grid_from_tilemap")]
pub fn og_nav_grid_from_tilemap(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_grid_from_tilemap(&Host, world, desc)
}

#[napi(js_name = "og_nav_set_cost")]
pub fn og_nav_set_cost(env: Env, world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_set_cost(&Host, world, grid, x, y, cost)
}

#[napi(js_name = "og_nav_cell_at")]
pub fn og_nav_cell_at(env: Env, world: u32, grid: f64, point: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_cell_at(&Host, world, grid, point)
}

#[napi(js_name = "og_nav_find_path")]
pub fn og_nav_find_path(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_find_path(&Host, world, desc)
}

#[napi(js_name = "og_nav_flow_field")]
pub fn og_nav_flow_field(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_flow_field(&Host, world, desc)
}

#[napi(js_name = "og_nav_set_flow_targets")]
pub fn og_nav_set_flow_targets(env: Env, world: u32, field: f64, targets: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_set_flow_targets(&Host, world, field, targets)
}

#[napi(js_name = "og_nav_flow_at")]
pub fn og_nav_flow_at(env: Env, world: u32, field: f64, point: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_flow_at(&Host, world, field, point)
}

#[napi(js_name = "og_nav_load")]
pub fn og_nav_load(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_load(&Host, world, path)
}

#[napi(js_name = "og_nav_build")]
pub fn og_nav_build(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_build(&Host, world, desc)
}

#[napi(js_name = "og_nav_set_obstacle")]
pub fn og_nav_set_obstacle(env: Env, world: u32, mesh: f64, id: u32, obstacle: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_set_obstacle(&Host, world, mesh, id, obstacle)
}

#[napi(js_name = "og_nav_nearest")]
pub fn og_nav_nearest(env: Env, world: u32, mesh: f64, point: String, extent: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_nearest(&Host, world, mesh, point, extent)
}

#[napi(js_name = "og_nav_path_of")]
pub fn og_nav_path_of(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as NavApi>::og_nav_path_of(&Host, world, entity)
}

#[napi(js_name = "og_ai_load")]
pub fn og_ai_load(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_load(&Host, world, path)
}

#[napi(js_name = "og_ai_parse")]
pub fn og_ai_parse(env: Env, world: u32, text: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_parse(&Host, world, text)
}

#[napi(js_name = "og_ai_get")]
pub fn og_ai_get(env: Env, world: u32, entity: f64, key: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_get(&Host, world, entity, key)
}

#[napi(js_name = "og_ai_set")]
pub fn og_ai_set(env: Env, world: u32, entity: f64, key: String, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_set(&Host, world, entity, key, value)
}

#[napi(js_name = "og_ai_noise")]
pub fn og_ai_noise(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_noise(&Host, world, desc)
}

#[napi(js_name = "og_ai_describe")]
pub fn og_ai_describe(env: Env, world: u32, entity: f64) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as AiApi>::og_ai_describe(&Host, world, entity)
}

#[napi(js_name = "og_platform_store_get")]
pub fn og_platform_store_get(env: Env, world: u32, key: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_store_get(&Host, world, key)
}

#[napi(js_name = "og_platform_store_set")]
pub fn og_platform_store_set(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_store_set(&Host, world, desc)
}

#[napi(js_name = "og_platform_store_list")]
pub fn og_platform_store_list(env: Env, world: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_store_list(&Host, world)
}

#[napi(js_name = "og_platform_leaderboard_submit")]
pub fn og_platform_leaderboard_submit(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_leaderboard_submit(&Host, world, desc)
}

#[napi(js_name = "og_platform_leaderboard_top")]
pub fn og_platform_leaderboard_top(env: Env, world: u32, desc: Option<String>) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_leaderboard_top(&Host, world, desc)
}

#[napi(js_name = "og_platform_leaderboard_around")]
pub fn og_platform_leaderboard_around(env: Env, world: u32, desc: Option<String>) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_leaderboard_around(&Host, world, desc)
}

#[napi(js_name = "og_platform_telemetry")]
pub fn og_platform_telemetry(env: Env, world: u32, name: String, attrs: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_telemetry(&Host, world, name, attrs)
}

#[napi(js_name = "og_platform_point")]
pub fn og_platform_point(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_point(&Host, world, desc)
}

#[napi(js_name = "og_platform_ai_mark")]
pub fn og_platform_ai_mark(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_ai_mark(&Host, world, desc)
}

#[napi(js_name = "og_platform_load_settings")]
pub fn og_platform_load_settings(env: Env, world: u32) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_load_settings(&Host, world)
}

#[napi(js_name = "og_platform_save_settings")]
pub fn og_platform_save_settings(env: Env, world: u32, value: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_save_settings(&Host, world, value)
}

#[napi(js_name = "og_platform_result")]
pub fn og_platform_result(env: Env, world: u32, request: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_result(&Host, world, request)
}

#[napi(js_name = "og_platform_info")]
pub fn og_platform_info(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as PlatformApi>::og_platform_info(&Host, world)
}

#[napi(js_name = "og_i18n_load")]
pub fn og_i18n_load(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as I18nApi>::og_i18n_load(&Host, world, path)
}

#[napi(js_name = "og_i18n_set_locale")]
pub fn og_i18n_set_locale(env: Env, world: u32, locale: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as I18nApi>::og_i18n_set_locale(&Host, world, locale)
}

#[napi(js_name = "og_i18n_locale")]
pub fn og_i18n_locale(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as I18nApi>::og_i18n_locale(&Host, world)
}

#[napi(js_name = "og_i18n_t")]
pub fn og_i18n_t(env: Env, world: u32, key: String, args: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as I18nApi>::og_i18n_t(&Host, world, key, args)
}

#[napi(js_name = "og_i18n_has")]
pub fn og_i18n_has(env: Env, world: u32, key: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as I18nApi>::og_i18n_has(&Host, world, key)
}

#[napi(js_name = "og_ui_configure")]
pub fn og_ui_configure(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_configure(&Host, world, desc)
}

#[napi(js_name = "og_ui_viewport")]
pub fn og_ui_viewport(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_viewport(&Host, world)
}

#[napi(js_name = "og_ui_theme")]
pub fn og_ui_theme(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_theme(&Host, world)
}

#[napi(js_name = "og_ui_open")]
pub fn og_ui_open(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open(&Host, id, desc)
}

#[napi(js_name = "og_ui_open_panel")]
pub fn og_ui_open_panel(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_panel(&Host, id, desc)
}

#[napi(js_name = "og_ui_open_scroll")]
pub fn og_ui_open_scroll(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_scroll(&Host, id, desc)
}

#[napi(js_name = "og_ui_close")]
pub fn og_ui_close(env: Env) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_close(&Host)
}

#[napi(js_name = "og_ui_label")]
pub fn og_ui_label(env: Env, id: String, text: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_label(&Host, id, text, desc)
}

#[napi(js_name = "og_ui_button")]
pub fn og_ui_button(env: Env, id: String, text: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_button(&Host, id, text, desc)
}

#[napi(js_name = "og_ui_image")]
pub fn og_ui_image(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_image(&Host, id, desc)
}

#[napi(js_name = "og_ui_progress")]
pub fn og_ui_progress(env: Env, id: String, value: f64, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_progress(&Host, id, value, desc)
}

#[napi(js_name = "og_ui_slider")]
pub fn og_ui_slider(env: Env, id: String, value: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_slider(&Host, id, value, desc)
}

#[napi(js_name = "og_ui_toggle")]
pub fn og_ui_toggle(env: Env, id: String, on: bool, label: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_toggle(&Host, id, on, label)
}

#[napi(js_name = "og_ui_text_field")]
pub fn og_ui_text_field(env: Env, id: String, value: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_text_field(&Host, id, value, desc)
}

#[napi(js_name = "og_ui_tooltip")]
pub fn og_ui_tooltip(env: Env, text: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_tooltip(&Host, text)
}

#[napi(js_name = "og_ui_breakpoint")]
pub fn og_ui_breakpoint(env: Env) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_breakpoint(&Host)
}

#[napi(js_name = "og_ui_frame_viewport")]
pub fn og_ui_frame_viewport(env: Env) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_frame_viewport(&Host)
}

#[napi(js_name = "og_ui_has_icon")]
pub fn og_ui_has_icon(env: Env, name: String) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_has_icon(&Host, name)
}

#[napi(js_name = "og_ui_seconds")]
pub fn og_ui_seconds(env: Env) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_seconds(&Host)
}

#[napi(js_name = "og_ui_frame_theme")]
pub fn og_ui_frame_theme(env: Env) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_frame_theme(&Host)
}

#[napi(js_name = "og_ui_open_anchor")]
pub fn og_ui_open_anchor(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_anchor(&Host, id, desc)
}

#[napi(js_name = "og_ui_open_transition")]
pub fn og_ui_open_transition(env: Env, id: String, show: bool, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_transition(&Host, id, show, desc)
}

#[napi(js_name = "og_ui_open_modal")]
pub fn og_ui_open_modal(env: Env, id: String, open: bool, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_modal(&Host, id, open, desc)
}

#[napi(js_name = "og_ui_close_modal")]
pub fn og_ui_close_modal(env: Env) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_close_modal(&Host)
}

#[napi(js_name = "og_ui_spotlight")]
pub fn og_ui_spotlight(env: Env, id: String, hole: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_spotlight(&Host, id, hole, desc)
}

#[napi(js_name = "og_ui_bounds")]
pub fn og_ui_bounds(env: Env, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_bounds(&Host, path)
}

#[napi(js_name = "og_ui_open_list")]
pub fn og_ui_open_list(env: Env, id: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_list(&Host, id, desc)
}

#[napi(js_name = "og_ui_open_grid")]
pub fn og_ui_open_grid(env: Env, id: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_open_grid(&Host, id, desc)
}

#[napi(js_name = "og_ui_list_item")]
pub fn og_ui_list_item(env: Env, index: u32, selected: bool) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_list_item(&Host, index, selected)
}

#[napi(js_name = "og_ui_grid_item")]
pub fn og_ui_grid_item(env: Env, index: u32, selected: bool) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_grid_item(&Host, index, selected)
}

#[napi(js_name = "og_ui_close_list")]
pub fn og_ui_close_list(env: Env) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_close_list(&Host)
}

#[napi(js_name = "og_ui_dragging")]
pub fn og_ui_dragging(env: Env, id: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_dragging(&Host, id)
}

#[napi(js_name = "og_ui_tabs")]
pub fn og_ui_tabs(env: Env, id: String, items: String, selected: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_tabs(&Host, id, items, selected, desc)
}

#[napi(js_name = "og_ui_select")]
pub fn og_ui_select(env: Env, id: String, options: String, selected: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_select(&Host, id, options, selected, desc)
}

#[napi(js_name = "og_ui_stepper")]
pub fn og_ui_stepper(env: Env, id: String, value: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_stepper(&Host, id, value, desc)
}

#[napi(js_name = "og_ui_radio")]
pub fn og_ui_radio(env: Env, id: String, options: String, selected: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_radio(&Host, id, options, selected, desc)
}

#[napi(js_name = "og_ui_checkbox")]
pub fn og_ui_checkbox(env: Env, id: String, on: bool, label: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_checkbox(&Host, id, on, label)
}

#[napi(js_name = "og_ui_checkbox_list")]
pub fn og_ui_checkbox_list(env: Env, id: String, labels: String, values: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_checkbox_list(&Host, id, labels, values)
}

#[napi(js_name = "og_ui_chip")]
pub fn og_ui_chip(env: Env, id: String, text: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_chip(&Host, id, text, desc)
}

#[napi(js_name = "og_ui_badge")]
pub fn og_ui_badge(env: Env, id: String, text: String, tone: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_badge(&Host, id, text, tone)
}

#[napi(js_name = "og_ui_divider")]
pub fn og_ui_divider(env: Env, id: String, vertical: bool) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_divider(&Host, id, vertical)
}

#[napi(js_name = "og_ui_spacer")]
pub fn og_ui_spacer(env: Env, id: String, size: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_spacer(&Host, id, size)
}

#[napi(js_name = "og_ui_key_value")]
pub fn og_ui_key_value(env: Env, id: String, key: String, value: String, mono: bool) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_key_value(&Host, id, key, value, mono)
}

#[napi(js_name = "og_ui_skeleton")]
pub fn og_ui_skeleton(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_skeleton(&Host, id, desc)
}

#[napi(js_name = "og_ui_spinner")]
pub fn og_ui_spinner(env: Env, id: String, size: f64) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_spinner(&Host, id, size)
}

#[napi(js_name = "og_ui_icon")]
pub fn og_ui_icon(env: Env, id: String, name: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_icon(&Host, id, name, desc)
}

#[napi(js_name = "og_ui_meter")]
pub fn og_ui_meter(env: Env, id: String, value: f64, max: f64, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_meter(&Host, id, value, max, desc)
}

#[napi(js_name = "og_ui_cooldown")]
pub fn og_ui_cooldown(env: Env, id: String, remaining: f64, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_cooldown(&Host, id, remaining, desc)
}

#[napi(js_name = "og_ui_minimap")]
pub fn og_ui_minimap(env: Env, id: String, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_minimap(&Host, id, desc)
}

#[napi(js_name = "og_ui_dialogue")]
pub fn og_ui_dialogue(env: Env, id: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_dialogue(&Host, id, desc)
}

#[napi(js_name = "og_ui_joystick")]
pub fn og_ui_joystick(env: Env, id: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_joystick(&Host, id, desc)
}

#[napi(js_name = "og_ui_dpad")]
pub fn og_ui_dpad(env: Env, id: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_dpad(&Host, id, desc)
}

#[napi(js_name = "og_ui_action_button")]
pub fn og_ui_action_button(env: Env, id: String, text: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_action_button(&Host, id, text, desc)
}

#[napi(js_name = "og_ui_toast")]
pub fn og_ui_toast(env: Env, world: u32, text: String, desc: Option<String>) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_toast(&Host, world, text, desc)
}

#[napi(js_name = "og_ui_dismiss_toast")]
pub fn og_ui_dismiss_toast(env: Env, world: u32, toast_id: u32) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_dismiss_toast(&Host, world, toast_id)
}

#[napi(js_name = "og_ui_navigate")]
pub fn og_ui_navigate(env: Env, world: u32, intent: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_navigate(&Host, world, intent)
}

#[napi(js_name = "og_ui_back")]
pub fn og_ui_back(env: Env) -> bool {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_back(&Host)
}

#[napi(js_name = "og_ui_set_navigation")]
pub fn og_ui_set_navigation(env: Env, on: bool) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_set_navigation(&Host, on)
}

#[napi(js_name = "og_ui_memory")]
pub fn og_ui_memory(env: Env, id: String, initial: f64) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_memory(&Host, id, initial)
}

#[napi(js_name = "og_ui_remember")]
pub fn og_ui_remember(env: Env, id: String, value: f64) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_remember(&Host, id, value)
}

#[napi(js_name = "og_ui_tick")]
pub fn og_ui_tick(env: Env) -> u32 {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_tick(&Host)
}

#[napi(js_name = "og_ui_counter")]
pub fn og_ui_counter(env: Env, id: String, value: f64, desc: Option<String>) -> f64 {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_counter(&Host, id, value, desc)
}

#[napi(js_name = "og_ui_burst")]
pub fn og_ui_burst(env: Env, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_burst(&Host, desc)
}

#[napi(js_name = "og_ui_shake")]
pub fn og_ui_shake(env: Env, desc: Option<String>) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_shake(&Host, desc)
}

#[napi(js_name = "og_ui_screen_transition")]
pub fn og_ui_screen_transition(env: Env, id: String, screen: String, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_screen_transition(&Host, id, screen, desc)
}

#[napi(js_name = "og_ui_preview_viewport")]
pub fn og_ui_preview_viewport(env: Env, world: u32, viewport: String) {
    let _env = crate::EnvScope::enter(&env);
    <Host as UiApi>::og_ui_preview_viewport(&Host, world, viewport)
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
#[napi(js_name = "og_commands_help")]
pub fn og_commands_help(env: Env, world: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as CommandsApi>::og_commands_help(&Host, world)
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

#[cfg(not(feature = "publish"))]
#[napi(js_name = "og_engine_probe_listen")]
pub fn og_engine_probe_listen(env: Env, engine_probe: u32, seconds: Option<f64>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as ProbeApi>::og_engine_probe_listen(&Host, engine_probe, seconds)
}

#[napi(js_name = "og_terrain_load")]
pub fn og_terrain_load(env: Env, world: u32, path: String) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_load(&Host, world, path)
}

#[napi(js_name = "og_terrain_sample")]
pub fn og_terrain_sample(env: Env, world: u32, desc: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_sample(&Host, world, desc)
}

#[napi(js_name = "og_terrain_bounds")]
pub fn og_terrain_bounds(env: Env, world: u32, data: f64, origin: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_bounds(&Host, world, data, origin)
}

#[napi(js_name = "og_terrain_edit")]
pub fn og_terrain_edit(env: Env, world: u32, data: f64, brush: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_edit(&Host, world, data, brush)
}

#[napi(js_name = "og_terrain_layer")]
pub fn og_terrain_layer(env: Env, world: u32, data: f64, index: u32) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_layer(&Host, world, data, index)
}

#[napi(js_name = "og_terrain_set_layer")]
pub fn og_terrain_set_layer(env: Env, world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    let _env = crate::EnvScope::enter(&env);
    <Host as TerrainApi>::og_terrain_set_layer(&Host, world, data, index, layer)
}
