// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The wasm-bindgen exports of every binding symbol: each calls the
//! host's implementation of its `og_api::host` trait.

#![allow(clippy::too_many_arguments)]

use crate::WebHost as Host;
use og_api::host::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn og_host_set_dispatcher(dispatcher: js_sys::Function) {
    <Host as Lifecycle>::og_host_set_dispatcher(&Host, dispatcher)
}

#[wasm_bindgen]
pub fn og_host_shared() -> u32 {
    <Host as Lifecycle>::og_host_shared(&Host)
}

#[wasm_bindgen]
pub fn og_engine_create(game: u32, options: String) -> String {
    <Host as Lifecycle>::og_engine_create(&Host, game, options)
}

#[wasm_bindgen]
pub fn og_engine_destroy(engine: u32) -> bool {
    <Host as Lifecycle>::og_engine_destroy(&Host, engine)
}

#[wasm_bindgen]
pub fn og_engine_advance(engine: u32, steps: u32) -> String {
    <Host as Lifecycle>::og_engine_advance(&Host, engine, steps)
}

#[wasm_bindgen]
pub fn og_engine_command(engine: u32, action: String, args: String) -> String {
    <Host as Lifecycle>::og_engine_command(&Host, engine, action, args)
}

#[wasm_bindgen]
pub fn og_engine_frame(engine: u32, elapsed_ms: f64) -> String {
    <Host as Lifecycle>::og_engine_frame(&Host, engine, elapsed_ms)
}

#[wasm_bindgen]
pub fn og_host_begin_bundle() {
    <Host as Lifecycle>::og_host_begin_bundle(&Host)
}

#[wasm_bindgen]
pub fn og_engine_call(engine: u32, frame: String) -> String {
    <Host as Lifecycle>::og_engine_call(&Host, engine, frame)
}

#[wasm_bindgen]
pub fn og_engine_take_outputs(engine: u32) -> String {
    <Host as Lifecycle>::og_engine_take_outputs(&Host, engine)
}

#[wasm_bindgen]
pub fn og_engine_take_log(engine: u32) -> String {
    <Host as Lifecycle>::og_engine_take_log(&Host, engine)
}

#[wasm_bindgen]
pub fn og_engine_write_resource(world: u32, resource: String, value: String) -> String {
    <Host as Lifecycle>::og_engine_write_resource(&Host, world, resource, value)
}

#[wasm_bindgen]
pub fn og_engine_resource_layout(world: u32, resource: String) -> String {
    <Host as Lifecycle>::og_engine_resource_layout(&Host, world, resource)
}

#[wasm_bindgen]
pub fn og_engine_read_resource(world: u32, resource: String) -> Vec<u8> {
    <Host as Lifecycle>::og_engine_read_resource(&Host, world, resource)
}

#[wasm_bindgen]
pub fn og_engine_write_resource_bytes(world: u32, resource: String, bytes: Vec<u8>) -> String {
    <Host as Lifecycle>::og_engine_write_resource_bytes(&Host, world, resource, bytes)
}

#[wasm_bindgen]
pub fn og_engine_component_id(world: u32, component: String) -> u32 {
    <Host as Lifecycle>::og_engine_component_id(&Host, world, component)
}

#[wasm_bindgen]
pub fn og_engine_component_layout(world: u32, component: String) -> String {
    <Host as Lifecycle>::og_engine_component_layout(&Host, world, component)
}

#[wasm_bindgen]
pub fn og_engine_read_component(world: u32, entity: f64, component: u32) -> u32 {
    <Host as Lifecycle>::og_engine_read_component(&Host, world, entity, component)
}

#[wasm_bindgen]
pub fn og_engine_write_component(world: u32, entity: f64, component: u32, epoch: u32) -> u32 {
    <Host as Lifecycle>::og_engine_write_component(&Host, world, entity, component, epoch)
}

#[wasm_bindgen]
pub fn og_engine_query_archetypes(query: u32, from: u32) -> u32 {
    <Host as Lifecycle>::og_engine_query_archetypes(&Host, query, from)
}

#[wasm_bindgen]
pub fn og_engine_view_column(view: u32, component: u32) -> Vec<u32> {
    <Host as Lifecycle>::og_engine_view_column(&Host, view, component)
}

#[wasm_bindgen]
pub fn og_engine_view_add_scaled(view: u32, target: u32, target_field: u32, source: u32, source_field: u32, scale: f64) -> String {
    <Host as Lifecycle>::og_engine_view_add_scaled(&Host, view, target, target_field, source, source_field, scale)
}

#[wasm_bindgen]
pub fn og_engine_ui_patch(world: u32, bytes: Vec<u8>) -> Vec<u8> {
    <Host as Lifecycle>::og_engine_ui_patch(&Host, world, bytes)
}

#[wasm_bindgen]
pub fn og_engine_inspector_panel(engine: u32) -> String {
    <Host as Lifecycle>::og_engine_inspector_panel(&Host, engine)
}

#[wasm_bindgen]
pub fn og_engine_inspector_event(engine: u32, event: String) -> bool {
    <Host as Lifecycle>::og_engine_inspector_event(&Host, engine, event)
}

#[wasm_bindgen]
pub fn og_platform_take(engine: u32) -> String {
    <Host as Lifecycle>::og_platform_take(&Host, engine)
}

#[wasm_bindgen]
pub fn og_platform_answer(engine: u32, request: u32, answer: String) -> bool {
    <Host as Lifecycle>::og_platform_answer(&Host, engine, request, answer)
}

#[wasm_bindgen]
pub fn og_platform_set_host(engine: u32, info: String) {
    <Host as Lifecycle>::og_platform_set_host(&Host, engine, info)
}

#[wasm_bindgen]
pub fn og_platform_notify(engine: u32, notice: String) -> bool {
    <Host as Lifecycle>::og_platform_notify(&Host, engine, notice)
}

#[wasm_bindgen]
pub fn og_room_take(engine: u32) -> String {
    <Host as Lifecycle>::og_room_take(&Host, engine)
}

#[wasm_bindgen]
pub fn og_room_deliver(engine: u32, event: String) -> bool {
    <Host as Lifecycle>::og_room_deliver(&Host, engine, event)
}

#[wasm_bindgen]
pub fn og_room_set_host(engine: u32, info: String) {
    <Host as Lifecycle>::og_room_set_host(&Host, engine, info)
}

#[wasm_bindgen]
pub fn og_shared_take(engine: u32) -> String {
    <Host as Lifecycle>::og_shared_take(&Host, engine)
}

#[wasm_bindgen]
pub fn og_shared_deliver(engine: u32, event: String) -> bool {
    <Host as Lifecycle>::og_shared_deliver(&Host, engine, event)
}

#[wasm_bindgen]
pub fn og_engine_set_content(engine: u32, base: String) {
    <Host as Lifecycle>::og_engine_set_content(&Host, engine, base)
}

#[wasm_bindgen]
pub fn og_engine_trace(engine: u32, on: bool) {
    <Host as Lifecycle>::og_engine_trace(&Host, engine, on)
}

#[wasm_bindgen]
pub fn og_engine_trace_take(engine: u32, last: bool) -> String {
    <Host as Lifecycle>::og_engine_trace_take(&Host, engine, last)
}

#[wasm_bindgen]
pub fn og_engine_perf(engine: u32, on: bool) {
    <Host as Lifecycle>::og_engine_perf(&Host, engine, on)
}

#[wasm_bindgen]
pub fn og_engine_perf_report(engine: u32, budgets: String) -> String {
    <Host as Lifecycle>::og_engine_perf_report(&Host, engine, budgets)
}

#[wasm_bindgen]
pub fn og_engine_perf_frames(engine: u32) -> String {
    <Host as Lifecycle>::og_engine_perf_frames(&Host, engine)
}

#[wasm_bindgen]
pub fn og_present_frame(engine: u32, now_ms: f64) -> String {
    <Host as Present>::og_present_frame(&Host, engine, now_ms)
}

#[wasm_bindgen]
pub fn og_present_resume(engine: u32) {
    <Host as Present>::og_present_resume(&Host, engine)
}

#[wasm_bindgen]
pub fn og_present_refresh(engine: u32, hz: f64) {
    <Host as Present>::og_present_refresh(&Host, engine, hz)
}

#[wasm_bindgen]
pub fn og_present_fullscreen(engine: u32, on: bool) {
    <Host as Present>::og_present_fullscreen(&Host, engine, on)
}

#[wasm_bindgen]
pub fn og_reflect_define_component(name: String, fields: String) -> String {
    <Host as ReflectApi>::og_reflect_define_component(&Host, name, fields)
}

#[wasm_bindgen]
pub fn og_reflect_define_resource(name: String, fields: String) -> String {
    <Host as ReflectApi>::og_reflect_define_resource(&Host, name, fields)
}

#[wasm_bindgen]
pub fn og_reflect_define_event(name: String, fields: String) -> String {
    <Host as ReflectApi>::og_reflect_define_event(&Host, name, fields)
}

#[wasm_bindgen]
pub fn og_save_edit_get_version(save_edit: u32) -> u32 {
    <Host as WorldApi>::og_save_edit_get_version(&Host, save_edit)
}

#[wasm_bindgen]
pub fn og_save_edit_types(save_edit: u32) -> String {
    <Host as WorldApi>::og_save_edit_types(&Host, save_edit)
}

#[wasm_bindgen]
pub fn og_save_edit_records(save_edit: u32, name: String) -> String {
    <Host as WorldApi>::og_save_edit_records(&Host, save_edit, name)
}

#[wasm_bindgen]
pub fn og_save_edit_set_records(save_edit: u32, name: String, desc: Option<String>) -> String {
    <Host as WorldApi>::og_save_edit_set_records(&Host, save_edit, name, desc)
}

#[wasm_bindgen]
pub fn og_save_edit_rename_type(save_edit: u32, from: String, to: String) -> String {
    <Host as WorldApi>::og_save_edit_rename_type(&Host, save_edit, from, to)
}

#[wasm_bindgen]
pub fn og_save_edit_rename_field(save_edit: u32, name: String, from: String, to: String) -> String {
    <Host as WorldApi>::og_save_edit_rename_field(&Host, save_edit, name, from, to)
}

#[wasm_bindgen]
pub fn og_save_edit_drop_type(save_edit: u32, name: String) -> String {
    <Host as WorldApi>::og_save_edit_drop_type(&Host, save_edit, name)
}

#[wasm_bindgen]
pub fn og_save_edit_spawn(save_edit: u32, desc: Option<String>) -> String {
    <Host as WorldApi>::og_save_edit_spawn(&Host, save_edit, desc)
}

#[wasm_bindgen]
pub fn og_save_edit_text(save_edit: u32) -> String {
    <Host as WorldApi>::og_save_edit_text(&Host, save_edit)
}

#[wasm_bindgen]
pub fn og_save_edit_close(save_edit: u32) {
    <Host as WorldApi>::og_save_edit_close(&Host, save_edit)
}

#[wasm_bindgen]
pub fn og_spatial_index_get_count(spatial_index: u32) -> u32 {
    <Host as WorldApi>::og_spatial_index_get_count(&Host, spatial_index)
}

#[wasm_bindgen]
pub fn og_spatial_index_within(spatial_index: u32, center: String, radius: f64) -> String {
    <Host as WorldApi>::og_spatial_index_within(&Host, spatial_index, center, radius)
}

#[wasm_bindgen]
pub fn og_spatial_index_nearest(spatial_index: u32, center: String, count: u32) -> String {
    <Host as WorldApi>::og_spatial_index_nearest(&Host, spatial_index, center, count)
}

#[wasm_bindgen]
pub fn og_world_get_tick(world: u32) -> u32 {
    <Host as WorldApi>::og_world_get_tick(&Host, world)
}

#[wasm_bindgen]
pub fn og_world_spawn(world: u32, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_spawn(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_world_despawn(world: u32, entity: f64) -> String {
    <Host as WorldApi>::og_world_despawn(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_world_alive(world: u32, entity: f64) -> bool {
    <Host as WorldApi>::og_world_alive(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_world_has(world: u32, entity: f64, component: String) -> bool {
    <Host as WorldApi>::og_world_has(&Host, world, entity, component)
}

#[wasm_bindgen]
pub fn og_world_insert(world: u32, entity: f64, component: String, value: Option<String>) -> String {
    <Host as WorldApi>::og_world_insert(&Host, world, entity, component, value)
}

#[wasm_bindgen]
pub fn og_world_remove(world: u32, entity: f64, component: String) -> String {
    <Host as WorldApi>::og_world_remove(&Host, world, entity, component)
}

#[wasm_bindgen]
pub fn og_world_get(world: u32, entity: f64, component: String) -> String {
    <Host as WorldApi>::og_world_get(&Host, world, entity, component)
}

#[wasm_bindgen]
pub fn og_world_set(world: u32, entity: f64, component: String, value: String) -> String {
    <Host as WorldApi>::og_world_set(&Host, world, entity, component, value)
}

#[wasm_bindgen]
pub fn og_world_resource(world: u32, resource: String) -> String {
    <Host as WorldApi>::og_world_resource(&Host, world, resource)
}

#[wasm_bindgen]
pub fn og_world_children(world: u32, entity: f64) -> String {
    <Host as WorldApi>::og_world_children(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_world_removed(world: u32, component: String) -> String {
    <Host as WorldApi>::og_world_removed(&Host, world, component)
}

#[wasm_bindgen]
pub fn og_world_spatial(world: u32, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_spatial(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_world_state(world: u32, name: String) -> String {
    <Host as WorldApi>::og_world_state(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_world_set_state(world: u32, name: String, value: String) -> String {
    <Host as WorldApi>::og_world_set_state(&Host, world, name, value)
}

#[wasm_bindgen]
pub fn og_world_find(world: u32, name: String) -> String {
    <Host as WorldApi>::og_world_find(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_world_prefab_of(world: u32, entity: f64) -> String {
    <Host as WorldApi>::og_world_prefab_of(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_world_name_of(world: u32, entity: f64) -> String {
    <Host as WorldApi>::og_world_name_of(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_world_set_name(world: u32, entity: f64, name: String) -> String {
    <Host as WorldApi>::og_world_set_name(&Host, world, entity, name)
}

#[wasm_bindgen]
pub fn og_world_query(world: u32, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_query(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_world_events(world: u32, event: String) -> String {
    <Host as WorldApi>::og_world_events(&Host, world, event)
}

#[wasm_bindgen]
pub fn og_world_emit(world: u32, event: String, payload: String) -> String {
    <Host as WorldApi>::og_world_emit(&Host, world, event, payload)
}

#[wasm_bindgen]
pub fn og_world_instantiate(world: u32, asset: f64, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_instantiate(&Host, world, asset, desc)
}

#[wasm_bindgen]
pub fn og_world_snapshot(world: u32, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_snapshot(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_world_open_save(world: u32, save: String) -> String {
    <Host as WorldApi>::og_world_open_save(&Host, world, save)
}

#[wasm_bindgen]
pub fn og_world_restore(world: u32, save: String, desc: Option<String>) -> String {
    <Host as WorldApi>::og_world_restore(&Host, world, save, desc)
}

#[wasm_bindgen]
pub fn og_query_count(query: u32) -> u32 {
    <Host as WorldApi>::og_query_count(&Host, query)
}

#[wasm_bindgen]
pub fn og_archetype_view_get_count(archetype_view: u32) -> u32 {
    <Host as WorldApi>::og_archetype_view_get_count(&Host, archetype_view)
}

#[wasm_bindgen]
pub fn og_archetype_view_get_tick(archetype_view: u32) -> u32 {
    <Host as WorldApi>::og_archetype_view_get_tick(&Host, archetype_view)
}

#[wasm_bindgen]
pub fn og_archetype_view_get_entities(archetype_view: u32) -> Vec<u32> {
    <Host as WorldApi>::og_archetype_view_get_entities(&Host, archetype_view)
}

#[wasm_bindgen]
pub fn og_archetype_view_get_rows(archetype_view: u32) -> Vec<u32> {
    <Host as WorldApi>::og_archetype_view_get_rows(&Host, archetype_view)
}

#[wasm_bindgen]
pub fn og_archetype_view_entity(archetype_view: u32, index: u32) -> f64 {
    <Host as WorldApi>::og_archetype_view_entity(&Host, archetype_view, index)
}

#[wasm_bindgen]
pub fn og_hook_get_name(hook: u32) -> String {
    <Host as SystemApi>::og_hook_get_name(&Host, hook)
}

#[wasm_bindgen]
pub fn og_system_define_hook(name: String, component: String, desc: String, run: u32) -> u32 {
    <Host as SystemApi>::og_system_define_hook(&Host, name, component, desc, run)
}

#[wasm_bindgen]
pub fn og_system_get_name(system: u32) -> String {
    <Host as SystemApi>::og_system_get_name(&Host, system)
}

#[wasm_bindgen]
pub fn og_system_get_reads(system: u32) -> String {
    <Host as SystemApi>::og_system_get_reads(&Host, system)
}

#[wasm_bindgen]
pub fn og_system_get_writes(system: u32) -> String {
    <Host as SystemApi>::og_system_get_writes(&Host, system)
}

#[wasm_bindgen]
pub fn og_system_define_system(name: String, desc: String, run: u32) -> u32 {
    <Host as SystemApi>::og_system_define_system(&Host, name, desc, run)
}

#[wasm_bindgen]
pub fn og_game_get_tick(game: u32) -> u32 {
    <Host as GameApi>::og_game_get_tick(&Host, game)
}

#[wasm_bindgen]
pub fn og_game_get_systems(game: u32) -> String {
    <Host as GameApi>::og_game_get_systems(&Host, game)
}

#[wasm_bindgen]
pub fn og_game_game(desc: Option<String>) -> u32 {
    <Host as GameApi>::og_game_game(&Host, desc)
}

#[wasm_bindgen]
pub fn og_transform_compose(position: String, quaternion: String, scale: String) -> String {
    <Host as TransformApi>::og_transform_compose(&Host, position, quaternion, scale)
}

#[wasm_bindgen]
pub fn og_transform_decompose(matrix: String) -> String {
    <Host as TransformApi>::og_transform_decompose(&Host, matrix)
}

#[wasm_bindgen]
pub fn og_transform_matrix_world(world: u32, entity: f64) -> String {
    <Host as TransformApi>::og_transform_matrix_world(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_transform_look_at(eye: String, target: String, up: Option<String>) -> String {
    <Host as TransformApi>::og_transform_look_at(&Host, eye, target, up)
}

#[wasm_bindgen]
pub fn og_transform_face(position: String, target: String, up: Option<String>) -> String {
    <Host as TransformApi>::og_transform_face(&Host, position, target, up)
}

#[wasm_bindgen]
pub fn og_transform_forward(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_forward(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_right(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_right(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_up(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_up(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_camera_forward(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_camera_forward(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_camera_right(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_camera_right(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_heading(quaternion: String) -> f64 {
    <Host as TransformApi>::og_transform_heading(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_from_heading(heading: f64) -> String {
    <Host as TransformApi>::og_transform_from_heading(&Host, heading)
}

#[wasm_bindgen]
pub fn og_transform_face2d(position: String, target: String) -> String {
    <Host as TransformApi>::og_transform_face2d(&Host, position, target)
}

#[wasm_bindgen]
pub fn og_transform_forward2d(quaternion: String) -> String {
    <Host as TransformApi>::og_transform_forward2d(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_heading2d(quaternion: String) -> f64 {
    <Host as TransformApi>::og_transform_heading2d(&Host, quaternion)
}

#[wasm_bindgen]
pub fn og_transform_from_heading2d(heading: f64) -> String {
    <Host as TransformApi>::og_transform_from_heading2d(&Host, heading)
}

#[wasm_bindgen]
pub fn og_transform_from_axis_angle(axis: String, angle: f64) -> String {
    <Host as TransformApi>::og_transform_from_axis_angle(&Host, axis, angle)
}

#[wasm_bindgen]
pub fn og_time_set_scale(world: u32, scale: f64) -> String {
    <Host as TimeApi>::og_time_set_scale(&Host, world, scale)
}

#[wasm_bindgen]
pub fn og_time_hit_stop(world: u32, ticks: u32) -> String {
    <Host as TimeApi>::og_time_hit_stop(&Host, world, ticks)
}

#[wasm_bindgen]
pub fn og_time_set_group_scale(world: u32, group: u32, scale: f64) -> String {
    <Host as TimeApi>::og_time_set_group_scale(&Host, world, group, scale)
}

#[wasm_bindgen]
pub fn og_time_group_dt(world: u32, entity: f64) -> f64 {
    <Host as TimeApi>::og_time_group_dt(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_rng_stream_get_seed(rng_stream: u32) -> u32 {
    <Host as RngApi>::og_rng_stream_get_seed(&Host, rng_stream)
}

#[wasm_bindgen]
pub fn og_rng_stream_next(rng_stream: u32) -> f64 {
    <Host as RngApi>::og_rng_stream_next(&Host, rng_stream)
}

#[wasm_bindgen]
pub fn og_rng_stream_range(rng_stream: u32, low: f64, high: f64) -> f64 {
    <Host as RngApi>::og_rng_stream_range(&Host, rng_stream, low, high)
}

#[wasm_bindgen]
pub fn og_rng_stream_integer(rng_stream: u32, low: i32, high: i32) -> i32 {
    <Host as RngApi>::og_rng_stream_integer(&Host, rng_stream, low, high)
}

#[wasm_bindgen]
pub fn og_rng_stream_chance(rng_stream: u32, p: f64) -> bool {
    <Host as RngApi>::og_rng_stream_chance(&Host, rng_stream, p)
}

#[wasm_bindgen]
pub fn og_rng_stream_weighted(rng_stream: u32, weights: String) -> i32 {
    <Host as RngApi>::og_rng_stream_weighted(&Host, rng_stream, weights)
}

#[wasm_bindgen]
pub fn og_rng_stream_permutation(rng_stream: u32, n: u32) -> String {
    <Host as RngApi>::og_rng_stream_permutation(&Host, rng_stream, n)
}

#[wasm_bindgen]
pub fn og_rng_stream_fork(rng_stream: u32, label: String) -> u32 {
    <Host as RngApi>::og_rng_stream_fork(&Host, rng_stream, label)
}

#[wasm_bindgen]
pub fn og_rng_next(world: u32) -> f64 {
    <Host as RngApi>::og_rng_next(&Host, world)
}

#[wasm_bindgen]
pub fn og_rng_range(world: u32, low: f64, high: f64) -> f64 {
    <Host as RngApi>::og_rng_range(&Host, world, low, high)
}

#[wasm_bindgen]
pub fn og_rng_integer(world: u32, low: i32, high: i32) -> i32 {
    <Host as RngApi>::og_rng_integer(&Host, world, low, high)
}

#[wasm_bindgen]
pub fn og_rng_chance(world: u32, p: f64) -> bool {
    <Host as RngApi>::og_rng_chance(&Host, world, p)
}

#[wasm_bindgen]
pub fn og_rng_weighted(world: u32, weights: String) -> i32 {
    <Host as RngApi>::og_rng_weighted(&Host, world, weights)
}

#[wasm_bindgen]
pub fn og_rng_permutation(world: u32, n: u32) -> String {
    <Host as RngApi>::og_rng_permutation(&Host, world, n)
}

#[wasm_bindgen]
pub fn og_rng_fork(world: u32, label: String) -> u32 {
    <Host as RngApi>::og_rng_fork(&Host, world, label)
}

#[wasm_bindgen]
pub fn og_rng_reset(world: u32) {
    <Host as RngApi>::og_rng_reset(&Host, world)
}

#[wasm_bindgen]
pub fn og_procgen_noise(desc: String, x: f64, y: f64, z: f64) -> f64 {
    <Host as ProcgenApi>::og_procgen_noise(&Host, desc, x, y, z)
}

#[wasm_bindgen]
pub fn og_procgen_noise_grad(desc: String, x: f64, y: f64, z: f64) -> String {
    <Host as ProcgenApi>::og_procgen_noise_grad(&Host, desc, x, y, z)
}

#[wasm_bindgen]
pub fn og_procgen_curl_noise(desc: String, x: f64, y: f64, z: f64) -> String {
    <Host as ProcgenApi>::og_procgen_curl_noise(&Host, desc, x, y, z)
}

#[wasm_bindgen]
pub fn og_procgen_worley(seed: u32, x: f64, y: f64, z: f64, three_d: bool, sites: Option<String>) -> String {
    <Host as ProcgenApi>::og_procgen_worley(&Host, seed, x, y, z, three_d, sites)
}

#[wasm_bindgen]
pub fn og_procgen_noise_grid(desc: Option<String>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_noise_grid(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_poisson_disk(desc: Option<Vec<u8>>) -> String {
    <Host as ProcgenApi>::og_procgen_poisson_disk(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_bsp(desc: Option<Vec<u8>>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_bsp(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_wfc(desc: Option<String>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_wfc(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_heightmap(desc: Option<Vec<u8>>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_heightmap(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_plant(desc: Option<Vec<u8>>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_plant(&Host, desc)
}

#[wasm_bindgen]
pub fn og_procgen_fracture(desc: Option<Vec<u8>>) -> Vec<u8> {
    <Host as ProcgenApi>::og_procgen_fracture(&Host, desc)
}

#[wasm_bindgen]
pub fn og_input_intent(world: u32, event: String, payload: String) -> String {
    <Host as InputApi>::og_input_intent(&Host, world, event, payload)
}

#[wasm_bindgen]
pub fn og_input_take_intents(world: u32) -> String {
    <Host as InputApi>::og_input_take_intents(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_replay_intent(world: u32, intent: String) -> String {
    <Host as InputApi>::og_input_replay_intent(&Host, world, intent)
}

#[wasm_bindgen]
pub fn og_input_define_action(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_define_action(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_actions(world: u32) -> String {
    <Host as InputApi>::og_input_actions(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_action(world: u32, name: String) -> String {
    <Host as InputApi>::og_input_action(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_consume(world: u32, name: String) -> bool {
    <Host as InputApi>::og_input_consume(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_is_action_down(world: u32, name: String) -> bool {
    <Host as InputApi>::og_input_is_action_down(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_is_action_just_pressed(world: u32, name: String) -> bool {
    <Host as InputApi>::og_input_is_action_just_pressed(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_is_action_just_released(world: u32, name: String) -> bool {
    <Host as InputApi>::og_input_is_action_just_released(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_is_action_long_pressed(world: u32, name: String) -> bool {
    <Host as InputApi>::og_input_is_action_long_pressed(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_action_value(world: u32, name: String) -> f64 {
    <Host as InputApi>::og_input_action_value(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_action_vector(world: u32, name: String) -> String {
    <Host as InputApi>::og_input_action_vector(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_input_move_relative(world: u32, name: String, camera: String) -> String {
    <Host as InputApi>::og_input_move_relative(&Host, world, name, camera)
}

#[wasm_bindgen]
pub fn og_input_set_action_maps(world: u32, action_maps: String) {
    <Host as InputApi>::og_input_set_action_maps(&Host, world, action_maps)
}

#[wasm_bindgen]
pub fn og_input_action_maps(world: u32) -> String {
    <Host as InputApi>::og_input_action_maps(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_rebind(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_rebind(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_conflicts(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_conflicts(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_reset_bindings(world: u32, action: Option<String>) -> String {
    <Host as InputApi>::og_input_reset_bindings(&Host, world, action)
}

#[wasm_bindgen]
pub fn og_input_set_toggle(world: u32, action: String, on: bool) -> String {
    <Host as InputApi>::og_input_set_toggle(&Host, world, action, on)
}

#[wasm_bindgen]
pub fn og_input_save_bindings(world: u32) -> String {
    <Host as InputApi>::og_input_save_bindings(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_load_bindings(world: u32, text: String) -> String {
    <Host as InputApi>::og_input_load_bindings(&Host, world, text)
}

#[wasm_bindgen]
pub fn og_input_start_capture(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_start_capture(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_capture(world: u32) -> String {
    <Host as InputApi>::og_input_capture(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_cancel_capture(world: u32) {
    <Host as InputApi>::og_input_cancel_capture(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_apply_capture(world: u32, policy: Option<String>) -> String {
    <Host as InputApi>::og_input_apply_capture(&Host, world, policy)
}

#[wasm_bindgen]
pub fn og_input_glyph(world: u32, action: String, device: Option<String>) -> String {
    <Host as InputApi>::og_input_glyph(&Host, world, action, device)
}

#[wasm_bindgen]
pub fn og_input_glyphs(world: u32, action: String, device: Option<String>) -> String {
    <Host as InputApi>::og_input_glyphs(&Host, world, action, device)
}

#[wasm_bindgen]
pub fn og_input_control_glyph(control: String, device: Option<String>) -> String {
    <Host as InputApi>::og_input_control_glyph(&Host, control, device)
}

#[wasm_bindgen]
pub fn og_input_device(world: u32) -> String {
    <Host as InputApi>::og_input_device(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_pad_device(world: u32, pad: u32) -> String {
    <Host as InputApi>::og_input_pad_device(&Host, world, pad)
}

#[wasm_bindgen]
pub fn og_input_set_deadzones(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_set_deadzones(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_set_sensitivity(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_set_sensitivity(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_sensitivity(world: u32, action: String, device: Option<String>) -> String {
    <Host as InputApi>::og_input_sensitivity(&Host, world, action, device)
}

#[wasm_bindgen]
pub fn og_input_set_keyboard_layout(labels: String) {
    <Host as InputApi>::og_input_set_keyboard_layout(&Host, labels)
}

#[wasm_bindgen]
pub fn og_input_keyboard_layout() -> String {
    <Host as InputApi>::og_input_keyboard_layout(&Host)
}

#[wasm_bindgen]
pub fn og_input_set_mouse(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_set_mouse(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_mouse(world: u32) -> String {
    <Host as InputApi>::og_input_mouse(&Host, world)
}

#[wasm_bindgen]
pub fn og_input_rumble(world: u32, desc: Option<String>) -> String {
    <Host as InputApi>::og_input_rumble(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_input_down(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_down(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_just_pressed(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_just_pressed(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_just_released(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_just_released(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_axis(world: u32, negative: String, positive: String) -> f64 {
    <Host as InputApi>::og_input_axis(&Host, world, negative, positive)
}

#[wasm_bindgen]
pub fn og_input_axis2(world: u32, left: String, right: String, down: String, up: String) -> String {
    <Host as InputApi>::og_input_axis2(&Host, world, left, right, down, up)
}

#[wasm_bindgen]
pub fn og_input_button_down(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_down(&Host, world, button)
}

#[wasm_bindgen]
pub fn og_input_button_just_pressed(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_just_pressed(&Host, world, button)
}

#[wasm_bindgen]
pub fn og_input_button_just_released(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_just_released(&Host, world, button)
}

#[wasm_bindgen]
pub fn og_pointers_down(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_down(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_just_pressed(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_just_pressed(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_just_released(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_just_released(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_position(world: u32, id: u32) -> String {
    <Host as InputApi>::og_pointers_position(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_start(world: u32, id: u32) -> String {
    <Host as InputApi>::og_pointers_start(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_count(world: u32) -> u32 {
    <Host as InputApi>::og_pointers_count(&Host, world)
}

#[wasm_bindgen]
pub fn og_gamepad_connected(world: u32, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_connected(&Host, world, pad)
}

#[wasm_bindgen]
pub fn og_gamepad_down(world: u32, button: String, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_down(&Host, world, button, pad)
}

#[wasm_bindgen]
pub fn og_gamepad_just_pressed(world: u32, button: String, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_just_pressed(&Host, world, button, pad)
}

#[wasm_bindgen]
pub fn og_gamepad_just_released(world: u32, button: String, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_just_released(&Host, world, button, pad)
}

#[wasm_bindgen]
pub fn og_gamepad_axis(world: u32, axis: String, pad: Option<u32>) -> f64 {
    <Host as InputApi>::og_gamepad_axis(&Host, world, axis, pad)
}

#[wasm_bindgen]
pub fn og_camera_look_at(world: u32, target: String) {
    <Host as CameraApi>::og_camera_look_at(&Host, world, target)
}

#[wasm_bindgen]
pub fn og_camera_world_to_screen(world: u32, point: String) -> String {
    <Host as CameraApi>::og_camera_world_to_screen(&Host, world, point)
}

#[wasm_bindgen]
pub fn og_camera_screen_to_ray(world: u32, x: f64, y: f64) -> String {
    <Host as CameraApi>::og_camera_screen_to_ray(&Host, world, x, y)
}

#[wasm_bindgen]
pub fn og_camera_shake(world: u32, trauma: f64) -> String {
    <Host as CameraApi>::og_camera_shake(&Host, world, trauma)
}

#[wasm_bindgen]
pub fn og_camera_punch(world: u32, amount: f64) -> String {
    <Host as CameraApi>::og_camera_punch(&Host, world, amount)
}

#[wasm_bindgen]
pub fn og_camera_kick(world: u32, direction: String, distance: f64) -> String {
    <Host as CameraApi>::og_camera_kick(&Host, world, direction, distance)
}

#[wasm_bindgen]
pub fn og_render_snap(world: u32, entity: f64) -> String {
    <Host as RenderApi>::og_render_snap(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_render_snap_camera(world: u32) {
    <Host as RenderApi>::og_render_snap_camera(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_compile(world: u32) {
    <Host as RenderApi>::og_render_compile(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_reveal(world: u32, entity: f64, seconds: f64) -> String {
    <Host as RenderApi>::og_render_reveal(&Host, world, entity, seconds)
}

#[wasm_bindgen]
pub fn og_render_ribbon(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_ribbon(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_sprite(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_sprite(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_floating_text(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_floating_text(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_measure_text(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_measure_text(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_box_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_box_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_plane_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_plane_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_sphere_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_sphere_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_standard_material(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_standard_material(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_update_material(world: u32, material: f64, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_update_material(&Host, world, material, desc)
}

#[wasm_bindgen]
pub fn og_render_shader_material(world: u32, shader: f64, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_shader_material(&Host, world, shader, desc)
}

#[wasm_bindgen]
pub fn og_render_set_material_shader(world: u32, material: f64, shader: String) -> String {
    <Host as RenderApi>::og_render_set_material_shader(&Host, world, material, shader)
}

#[wasm_bindgen]
pub fn og_render_set_shader_param(world: u32, material: f64, name: String, value: String) -> String {
    <Host as RenderApi>::og_render_set_shader_param(&Host, world, material, name, value)
}

#[wasm_bindgen]
pub fn og_render_apply_look(world: u32, look: f64) -> String {
    <Host as RenderApi>::og_render_apply_look(&Host, world, look)
}

#[wasm_bindgen]
pub fn og_render_apply_quality(world: u32, preset: String) -> String {
    <Host as RenderApi>::og_render_apply_quality(&Host, world, preset)
}

#[wasm_bindgen]
pub fn og_render_set_display(world: u32, settings: Option<String>) -> String {
    <Host as RenderApi>::og_render_set_display(&Host, world, settings)
}

#[wasm_bindgen]
pub fn og_render_reset_display(world: u32) -> String {
    <Host as RenderApi>::og_render_reset_display(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_display(world: u32) -> String {
    <Host as RenderApi>::og_render_display(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_look(world: u32) -> String {
    <Host as RenderApi>::og_render_look(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_capabilities(world: u32) -> String {
    <Host as RenderApi>::og_render_capabilities(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_text(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_text(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_buffer_geometry(world: u32, desc: Option<Vec<u8>>) -> String {
    <Host as RenderApi>::og_render_buffer_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_update_geometry(world: u32, geometry: f64, desc: Option<Vec<u8>>) -> String {
    <Host as RenderApi>::og_render_update_geometry(&Host, world, geometry, desc)
}

#[wasm_bindgen]
pub fn og_render_geometry_info(world: u32, geometry: f64) -> String {
    <Host as RenderApi>::og_render_geometry_info(&Host, world, geometry)
}

#[wasm_bindgen]
pub fn og_render_cylinder_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_cylinder_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_cone_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_cone_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_capsule_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_capsule_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_torus_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_torus_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_ring_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_ring_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_circle_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_circle_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_tube_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_tube_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_shape_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_shape_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_extrude_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_extrude_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_lathe_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_lathe_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_polyhedron_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_polyhedron_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_icosahedron_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_icosahedron_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_dodecahedron_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_dodecahedron_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_octahedron_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_octahedron_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_tetrahedron_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_tetrahedron_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_sweep_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_sweep_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_csg_geometry(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_csg_geometry(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_merge_geometries(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_merge_geometries(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_lines(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_lines(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_points(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_points(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_raycast(world: u32, ray: String, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_raycast(&Host, world, ray, desc)
}

#[wasm_bindgen]
pub fn og_render_light2d_at(world: u32, point: String, layers: Option<u32>) -> String {
    <Host as RenderApi>::og_render_light2d_at(&Host, world, point, layers)
}

#[wasm_bindgen]
pub fn og_render_render_target(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_render_target(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_set_post_param(world: u32, post: f64, name: String, value: String) -> String {
    <Host as RenderApi>::og_render_set_post_param(&Host, world, post, name, value)
}

#[wasm_bindgen]
pub fn og_render_post_errors(world: u32) -> String {
    <Host as RenderApi>::og_render_post_errors(&Host, world)
}

#[wasm_bindgen]
pub fn og_render_baked_lighting(world: u32) -> String {
    <Host as RenderApi>::og_render_baked_lighting(&Host, world)
}

#[wasm_bindgen]
pub fn og_load_field(world: u32, path: String) -> Vec<u8> {
    <Host as LoadApi>::og_load_field(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_bundle(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_bundle(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_bundle_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_bundle_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_progress(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_progress(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_default_bundle(world: u32) -> String {
    <Host as LoadApi>::og_load_default_bundle(&Host, world)
}

#[wasm_bindgen]
pub fn og_load_gltf(world: u32, path: String, desc: Option<String>) -> String {
    <Host as LoadApi>::og_load_gltf(&Host, world, path, desc)
}

#[wasm_bindgen]
pub fn og_load_scene(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_scene(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_prefab(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_prefab(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_font(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_font(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_font_info(world: u32, font: f64) -> String {
    <Host as LoadApi>::og_load_font_info(&Host, world, font)
}

#[wasm_bindgen]
pub fn og_load_sound(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_sound(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_lut(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_lut(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_environment(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_environment(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_environment_sun(world: u32, map: f64, rotation: Option<f64>, intensity: Option<f64>) -> String {
    <Host as LoadApi>::og_load_environment_sun(&Host, world, map, rotation, intensity)
}

#[wasm_bindgen]
pub fn og_load_emitter(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_emitter(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_effect(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_effect(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_shader(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_shader(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_post(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_post(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_vector(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_vector(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_ui_material(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_ui_material(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_texture(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_texture(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_texture_array(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_texture_array(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_style(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_style(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_look(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_look(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_look_info(world: u32, look: f64) -> String {
    <Host as LoadApi>::og_load_look_info(&Host, world, look)
}

#[wasm_bindgen]
pub fn og_load_palette(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_palette(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_sound_info(world: u32, sound: f64) -> String {
    <Host as LoadApi>::og_load_sound_info(&Host, world, sound)
}

#[wasm_bindgen]
pub fn og_load_status(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_status(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_error(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_error(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_gltf_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_gltf_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_animation(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_animation(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_animgraph(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_animgraph(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_rig(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_rig(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_flipbook(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_flipbook(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_tilemap(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_tilemap(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_video(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_video(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_video_info(world: u32, video: f64) -> String {
    <Host as LoadApi>::og_load_video_info(&Host, world, video)
}

#[wasm_bindgen]
pub fn og_load_timeline(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_timeline(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_vfx(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_vfx(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_rig2d(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_rig2d(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_image_data(world: u32, name: String, bytes: Vec<u8>, desc: Option<String>) -> String {
    <Host as LoadApi>::og_load_image_data(&Host, world, name, bytes, desc)
}

#[wasm_bindgen]
pub fn og_load_gltf_data(world: u32, name: String, bytes: Vec<u8>, desc: Option<String>) -> String {
    <Host as LoadApi>::og_load_gltf_data(&Host, world, name, bytes, desc)
}

#[wasm_bindgen]
pub fn og_load_environment_data(world: u32, name: String, bytes: Vec<u8>, desc: Option<String>) -> String {
    <Host as LoadApi>::og_load_environment_data(&Host, world, name, bytes, desc)
}

#[wasm_bindgen]
pub fn og_load_data(world: u32, desc: Option<String>) -> String {
    <Host as LoadApi>::og_load_data(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_load_data_json(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_data_json(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_data_bytes(world: u32, asset: f64) -> Vec<u8> {
    <Host as LoadApi>::og_load_data_bytes(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_pack(world: u32, entry: Option<String>) -> String {
    <Host as LoadApi>::og_load_pack(&Host, world, entry)
}

#[wasm_bindgen]
pub fn og_load_environment_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_environment_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_release(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_release(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_image_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_image_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_completions(world: u32) -> String {
    <Host as LoadApi>::og_load_completions(&Host, world)
}

#[wasm_bindgen]
pub fn og_load_replay_completions(world: u32, completions: String) -> String {
    <Host as LoadApi>::og_load_replay_completions(&Host, world, completions)
}

#[wasm_bindgen]
pub fn og_physics_resize_character(world: u32, entity: f64, height: f64, radius: f64) -> String {
    <Host as PhysicsApi>::og_physics_resize_character(&Host, world, entity, height, radius)
}

#[wasm_bindgen]
pub fn og_physics_water(world: u32, point: String) -> String {
    <Host as PhysicsApi>::og_physics_water(&Host, world, point)
}

#[wasm_bindgen]
pub fn og_physics_water_height(world: u32, entity: f64, x: f64, z: f64) -> String {
    <Host as PhysicsApi>::og_physics_water_height(&Host, world, entity, x, z)
}

#[wasm_bindgen]
pub fn og_physics_cast_ray(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_cast_ray(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_cast_ray_all(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_cast_ray_all(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_project_point(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_project_point(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_cast_shape(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_cast_shape(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_cast_shape_all(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_cast_shape_all(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_overlap(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_overlap(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_overlap_point(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_overlap_point(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_mass_properties(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_mass_properties(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_velocity_at_point(world: u32, entity: f64, point: String) -> String {
    <Host as PhysicsApi>::og_physics_velocity_at_point(&Host, world, entity, point)
}

#[wasm_bindgen]
pub fn og_physics_sensor_contents(world: u32, sensor: f64) -> String {
    <Host as PhysicsApi>::og_physics_sensor_contents(&Host, world, sensor)
}

#[wasm_bindgen]
pub fn og_physics_apply_force(world: u32, entity: f64, force: String, point: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_apply_force(&Host, world, entity, force, point)
}

#[wasm_bindgen]
pub fn og_physics_apply_impulse(world: u32, entity: f64, impulse: String, point: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_apply_impulse(&Host, world, entity, impulse, point)
}

#[wasm_bindgen]
pub fn og_physics_apply_torque(world: u32, entity: f64, torque: String) -> String {
    <Host as PhysicsApi>::og_physics_apply_torque(&Host, world, entity, torque)
}

#[wasm_bindgen]
pub fn og_physics_apply_torque_impulse(world: u32, entity: f64, torque: String) -> String {
    <Host as PhysicsApi>::og_physics_apply_torque_impulse(&Host, world, entity, torque)
}

#[wasm_bindgen]
pub fn og_physics_set_mesh(world: u32, mesh: f64, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_set_mesh(&Host, world, mesh, desc)
}

#[wasm_bindgen]
pub fn og_physics_rope_points(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_rope_points(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_garment_points(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_garment_points(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_cloth_points(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_cloth_points(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_soft_body_points(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_soft_body_points(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_fluid_particles(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_fluid_particles(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_fluid3d_particles(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_fluid3d_particles(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_vehicle_wheels(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_vehicle_wheels(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_ragdoll_rig_from_skeleton(world: u32, skeleton: f64, bones: String) -> String {
    <Host as PhysicsApi>::og_physics_ragdoll_rig_from_skeleton(&Host, world, skeleton, bones)
}

#[wasm_bindgen]
pub fn og_physics_set_ragdoll_rig(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_set_ragdoll_rig(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_physics_ragdoll_pose(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_ragdoll_pose(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_apply_ragdoll_impulse(world: u32, entity: f64, bone_index: u32, impulse: String, point: String) -> String {
    <Host as PhysicsApi>::og_physics_apply_ragdoll_impulse(&Host, world, entity, bone_index, impulse, point)
}

#[wasm_bindgen]
pub fn og_physics_stats(world: u32) -> String {
    <Host as PhysicsApi>::og_physics_stats(&Host, world)
}

#[wasm_bindgen]
pub fn og_atmosphere_time_of_day(world: u32) -> f64 {
    <Host as AtmosphereApi>::og_atmosphere_time_of_day(&Host, world)
}

#[wasm_bindgen]
pub fn og_atmosphere_season(world: u32) -> String {
    <Host as AtmosphereApi>::og_atmosphere_season(&Host, world)
}

#[wasm_bindgen]
pub fn og_anim_get_node_matrix(world: u32, entity: f64, node: u32) -> String {
    <Host as AnimApi>::og_anim_get_node_matrix(&Host, world, entity, node)
}

#[wasm_bindgen]
pub fn og_anim_graph(world: u32, desc: Option<String>) -> String {
    <Host as AnimApi>::og_anim_graph(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_anim_humanoid(world: u32, desc: Option<String>) -> String {
    <Host as AnimApi>::og_anim_humanoid(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_anim_retarget(world: u32, desc: Option<String>) -> String {
    <Host as AnimApi>::og_anim_retarget(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_anim_mirror(world: u32, desc: Option<String>) -> String {
    <Host as AnimApi>::og_anim_mirror(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_anim_rig2d_info(world: u32, rig: f64) -> String {
    <Host as AnimApi>::og_anim_rig2d_info(&Host, world, rig)
}

#[wasm_bindgen]
pub fn og_anim_skeleton2d_boxes(world: u32, entity: f64) -> String {
    <Host as AnimApi>::og_anim_skeleton2d_boxes(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_tween_timeline_get_tween(tween_timeline: u32) -> f64 {
    <Host as TweenApi>::og_tween_timeline_get_tween(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_get_error(tween_timeline: u32) -> String {
    <Host as TweenApi>::og_tween_timeline_get_error(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_to(tween_timeline: u32, targets: String, vars: Option<String>, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_to(&Host, tween_timeline, targets, vars, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_from(tween_timeline: u32, targets: String, vars: Option<String>, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_from(&Host, tween_timeline, targets, vars, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_from_to(tween_timeline: u32, targets: String, vars: Option<String>, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_from_to(&Host, tween_timeline, targets, vars, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_set(tween_timeline: u32, targets: String, vars: Option<String>, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_set(&Host, tween_timeline, targets, vars, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_call(tween_timeline: u32, name: String, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_call(&Host, tween_timeline, name, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_add_label(tween_timeline: u32, label: String, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_add_label(&Host, tween_timeline, label, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_add(tween_timeline: u32, child: String, position: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_add(&Host, tween_timeline, child, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_play(tween_timeline: u32, from: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_play(&Host, tween_timeline, from)
}

#[wasm_bindgen]
pub fn og_tween_timeline_pause(tween_timeline: u32, at: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_pause(&Host, tween_timeline, at)
}

#[wasm_bindgen]
pub fn og_tween_timeline_resume(tween_timeline: u32) -> u32 {
    <Host as TweenApi>::og_tween_timeline_resume(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_reverse(tween_timeline: u32, from: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline_reverse(&Host, tween_timeline, from)
}

#[wasm_bindgen]
pub fn og_tween_timeline_restart(tween_timeline: u32) -> u32 {
    <Host as TweenApi>::og_tween_timeline_restart(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_seek(tween_timeline: u32, position: String) -> u32 {
    <Host as TweenApi>::og_tween_timeline_seek(&Host, tween_timeline, position)
}

#[wasm_bindgen]
pub fn og_tween_timeline_progress(tween_timeline: u32, value: Option<f64>) -> f64 {
    <Host as TweenApi>::og_tween_timeline_progress(&Host, tween_timeline, value)
}

#[wasm_bindgen]
pub fn og_tween_timeline_time(tween_timeline: u32) -> f64 {
    <Host as TweenApi>::og_tween_timeline_time(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_duration(tween_timeline: u32) -> f64 {
    <Host as TweenApi>::og_tween_timeline_duration(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_time_scale(tween_timeline: u32, value: Option<f64>) -> f64 {
    <Host as TweenApi>::og_tween_timeline_time_scale(&Host, tween_timeline, value)
}

#[wasm_bindgen]
pub fn og_tween_timeline_is_active(tween_timeline: u32) -> bool {
    <Host as TweenApi>::og_tween_timeline_is_active(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_timeline_kill(tween_timeline: u32) -> bool {
    <Host as TweenApi>::og_tween_timeline_kill(&Host, tween_timeline)
}

#[wasm_bindgen]
pub fn og_tween_to(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_to(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_from(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_from(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_from_to(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_from_to(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_sequence(world: u32, steps: String, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_sequence(&Host, world, steps, desc)
}

#[wasm_bindgen]
pub fn og_tween_parallel(world: u32, tweens: String, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_parallel(&Host, world, tweens, desc)
}

#[wasm_bindgen]
pub fn og_tween_timeline(world: u32, desc: Option<String>) -> u32 {
    <Host as TweenApi>::og_tween_timeline(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_cancel(world: u32, tween: f64) -> bool {
    <Host as TweenApi>::og_tween_cancel(&Host, world, tween)
}

#[wasm_bindgen]
pub fn og_tween_cancel_entity(world: u32, entity: f64) -> u32 {
    <Host as TweenApi>::og_tween_cancel_entity(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_tween_active(world: u32, tween: f64) -> bool {
    <Host as TweenApi>::og_tween_active(&Host, world, tween)
}

#[wasm_bindgen]
pub fn og_tween_squash(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_squash(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_flash(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_flash(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tilemap_set_tile(world: u32, desc: Option<String>) -> String {
    <Host as TilemapApi>::og_tilemap_set_tile(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tilemap_fill(world: u32, desc: Option<Vec<u8>>) -> String {
    <Host as TilemapApi>::og_tilemap_fill(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tilemap_create(world: u32, desc: Option<String>) -> String {
    <Host as TilemapApi>::og_tilemap_create(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tilemap_resize(world: u32, desc: Option<String>) -> String {
    <Host as TilemapApi>::og_tilemap_resize(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tilemap_neighbours(world: u32, map: f64, x: u32, y: u32) -> String {
    <Host as TilemapApi>::og_tilemap_neighbours(&Host, world, map, x, y)
}

#[wasm_bindgen]
pub fn og_tilemap_get_tile(world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32 {
    <Host as TilemapApi>::og_tilemap_get_tile(&Host, world, entity, layer, x, y)
}

#[wasm_bindgen]
pub fn og_tilemap_cell_at(world: u32, entity: f64, point: String) -> String {
    <Host as TilemapApi>::og_tilemap_cell_at(&Host, world, entity, point)
}

#[wasm_bindgen]
pub fn og_tilemap_cell_center(world: u32, entity: f64, x: u32, y: u32) -> String {
    <Host as TilemapApi>::og_tilemap_cell_center(&Host, world, entity, x, y)
}

#[wasm_bindgen]
pub fn og_audio_play_feedback(world: u32, sound: f64, desc: Option<String>) -> String {
    <Host as AudioApi>::og_audio_play_feedback(&Host, world, sound, desc)
}

#[wasm_bindgen]
pub fn og_audio_create_sound(world: u32, desc: Option<Vec<u8>>) -> String {
    <Host as AudioApi>::og_audio_create_sound(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_audio_synth(desc: Option<String>) -> Vec<u8> {
    <Host as AudioApi>::og_audio_synth(&Host, desc)
}

#[wasm_bindgen]
pub fn og_audio_create_synth_sound(world: u32, desc: Option<String>) -> String {
    <Host as AudioApi>::og_audio_create_synth_sound(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_audio_output_latency(world: u32) -> f64 {
    <Host as AudioApi>::og_audio_output_latency(&Host, world)
}

#[wasm_bindgen]
pub fn og_nav_grid(world: u32, desc: Option<Vec<u8>>) -> String {
    <Host as NavApi>::og_nav_grid(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_nav_grid_from_tilemap(world: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_grid_from_tilemap(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_nav_set_cost(world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String {
    <Host as NavApi>::og_nav_set_cost(&Host, world, grid, x, y, cost)
}

#[wasm_bindgen]
pub fn og_nav_cell_at(world: u32, grid: f64, point: String) -> String {
    <Host as NavApi>::og_nav_cell_at(&Host, world, grid, point)
}

#[wasm_bindgen]
pub fn og_nav_find_path(world: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_find_path(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_nav_flow_field(world: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_flow_field(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_nav_set_flow_targets(world: u32, field: f64, targets: String) -> String {
    <Host as NavApi>::og_nav_set_flow_targets(&Host, world, field, targets)
}

#[wasm_bindgen]
pub fn og_nav_flow_at(world: u32, field: f64, point: String) -> String {
    <Host as NavApi>::og_nav_flow_at(&Host, world, field, point)
}

#[wasm_bindgen]
pub fn og_nav_load(world: u32, path: String, profile: Option<String>) -> String {
    <Host as NavApi>::og_nav_load(&Host, world, path, profile)
}

#[wasm_bindgen]
pub fn og_nav_build(world: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_build(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_nav_set_obstacle(world: u32, mesh: f64, id: u32, obstacle: Option<String>) -> String {
    <Host as NavApi>::og_nav_set_obstacle(&Host, world, mesh, id, obstacle)
}

#[wasm_bindgen]
pub fn og_nav_carve(world: u32, mesh: f64, id: u32, shape: Option<String>) -> String {
    <Host as NavApi>::og_nav_carve(&Host, world, mesh, id, shape)
}

#[wasm_bindgen]
pub fn og_nav_set_area(world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_set_area(&Host, world, mesh, id, desc)
}

#[wasm_bindgen]
pub fn og_nav_set_link(world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
    <Host as NavApi>::og_nav_set_link(&Host, world, mesh, id, desc)
}

#[wasm_bindgen]
pub fn og_nav_set_link_open(world: u32, mesh: f64, id: u32, open: bool) -> String {
    <Host as NavApi>::og_nav_set_link_open(&Host, world, mesh, id, open)
}

#[wasm_bindgen]
pub fn og_nav_finish_link(world: u32, entity: f64) -> bool {
    <Host as NavApi>::og_nav_finish_link(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_nav_nearest(world: u32, mesh: f64, point: String, extent: Option<String>) -> String {
    <Host as NavApi>::og_nav_nearest(&Host, world, mesh, point, extent)
}

#[wasm_bindgen]
pub fn og_nav_path_of(world: u32, entity: f64) -> String {
    <Host as NavApi>::og_nav_path_of(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_ai_load(world: u32, path: String) -> String {
    <Host as AiApi>::og_ai_load(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_ai_parse(world: u32, text: String) -> String {
    <Host as AiApi>::og_ai_parse(&Host, world, text)
}

#[wasm_bindgen]
pub fn og_ai_get(world: u32, entity: f64, key: String) -> String {
    <Host as AiApi>::og_ai_get(&Host, world, entity, key)
}

#[wasm_bindgen]
pub fn og_ai_set(world: u32, entity: f64, key: String, value: String) -> String {
    <Host as AiApi>::og_ai_set(&Host, world, entity, key, value)
}

#[wasm_bindgen]
pub fn og_ai_restart(world: u32, entity: f64) -> String {
    <Host as AiApi>::og_ai_restart(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_ai_finish(world: u32, entity: f64, success: bool, action: Option<String>) -> bool {
    <Host as AiApi>::og_ai_finish(&Host, world, entity, success, action)
}

#[wasm_bindgen]
pub fn og_ai_noise(world: u32, desc: Option<String>) -> String {
    <Host as AiApi>::og_ai_noise(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_ai_describe(world: u32, entity: f64) -> String {
    <Host as AiApi>::og_ai_describe(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_story_set_saliency(world: u32, strategy: String) -> String {
    <Host as StoryApi>::og_story_set_saliency(&Host, world, strategy)
}

#[wasm_bindgen]
pub fn og_story_load(world: u32, path: String) -> String {
    <Host as StoryApi>::og_story_load(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_story_parse(world: u32, text: String) -> String {
    <Host as StoryApi>::og_story_parse(&Host, world, text)
}

#[wasm_bindgen]
pub fn og_story_start(world: u32, desc: Option<String>) -> String {
    <Host as StoryApi>::og_story_start(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_story_view(world: u32, conversation: u32) -> String {
    <Host as StoryApi>::og_story_view(&Host, world, conversation)
}

#[wasm_bindgen]
pub fn og_story_graph(world: u32, dialogue: String) -> String {
    <Host as StoryApi>::og_story_graph(&Host, world, dialogue)
}

#[wasm_bindgen]
pub fn og_story_history(world: u32, conversation: u32) -> String {
    <Host as StoryApi>::og_story_history(&Host, world, conversation)
}

#[wasm_bindgen]
pub fn og_story_conversations(world: u32) -> String {
    <Host as StoryApi>::og_story_conversations(&Host, world)
}

#[wasm_bindgen]
pub fn og_story_advance(world: u32, conversation: u32) -> String {
    <Host as StoryApi>::og_story_advance(&Host, world, conversation)
}

#[wasm_bindgen]
pub fn og_story_choose(world: u32, conversation: u32, index: u32) -> String {
    <Host as StoryApi>::og_story_choose(&Host, world, conversation, index)
}

#[wasm_bindgen]
pub fn og_story_stop(world: u32, conversation: u32) -> bool {
    <Host as StoryApi>::og_story_stop(&Host, world, conversation)
}

#[wasm_bindgen]
pub fn og_story_bark(world: u32, desc: Option<String>) -> String {
    <Host as StoryApi>::og_story_bark(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_story_ambient(world: u32, desc: Option<String>) -> String {
    <Host as StoryApi>::og_story_ambient(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_story_barks(world: u32) -> String {
    <Host as StoryApi>::og_story_barks(&Host, world)
}

#[wasm_bindgen]
pub fn og_story_get(world: u32, name: String) -> String {
    <Host as StoryApi>::og_story_get(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_story_set(world: u32, name: String, value: String) -> String {
    <Host as StoryApi>::og_story_set(&Host, world, name, value)
}

#[wasm_bindgen]
pub fn og_story_reset(world: u32) {
    <Host as StoryApi>::og_story_reset(&Host, world)
}

#[wasm_bindgen]
pub fn og_story_start_quest(world: u32, id: String) -> String {
    <Host as StoryApi>::og_story_start_quest(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_story_progress(world: u32, quest: String, objective: String, amount: Option<u32>) -> bool {
    <Host as StoryApi>::og_story_progress(&Host, world, quest, objective, amount)
}

#[wasm_bindgen]
pub fn og_story_set_stage(world: u32, quest: String, stage: String) -> String {
    <Host as StoryApi>::og_story_set_stage(&Host, world, quest, stage)
}

#[wasm_bindgen]
pub fn og_story_complete_quest(world: u32, quest: String) -> String {
    <Host as StoryApi>::og_story_complete_quest(&Host, world, quest)
}

#[wasm_bindgen]
pub fn og_story_fail_quest(world: u32, quest: String) -> String {
    <Host as StoryApi>::og_story_fail_quest(&Host, world, quest)
}

#[wasm_bindgen]
pub fn og_story_quest(world: u32, id: String) -> String {
    <Host as StoryApi>::og_story_quest(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_story_quests(world: u32) -> String {
    <Host as StoryApi>::og_story_quests(&Host, world)
}

#[wasm_bindgen]
pub fn og_story_track(world: u32, id: String, on: bool) -> bool {
    <Host as StoryApi>::og_story_track(&Host, world, id, on)
}

#[wasm_bindgen]
pub fn og_story_mark_seen(world: u32, id: String) -> bool {
    <Host as StoryApi>::og_story_mark_seen(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_story_mark(world: u32, id: String, desc: Option<String>) -> bool {
    <Host as StoryApi>::og_story_mark(&Host, world, id, desc)
}

#[wasm_bindgen]
pub fn og_platform_store_get(world: u32, key: String) -> String {
    <Host as PlatformApi>::og_platform_store_get(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_platform_store_set(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_store_set(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_catalog_head(world: u32, catalog: String) -> String {
    <Host as PlatformApi>::og_platform_catalog_head(&Host, world, catalog)
}

#[wasm_bindgen]
pub fn og_platform_store_list(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_store_list(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_leaderboard_submit(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_leaderboard_submit(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_leaderboard_top(world: u32, desc: Option<String>) -> u32 {
    <Host as PlatformApi>::og_platform_leaderboard_top(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_leaderboard_around(world: u32, desc: Option<String>) -> u32 {
    <Host as PlatformApi>::og_platform_leaderboard_around(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_telemetry(world: u32, name: String, attrs: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_telemetry(&Host, world, name, attrs)
}

#[wasm_bindgen]
pub fn og_platform_point(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_point(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_ai_mark(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_ai_mark(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_load_settings(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_load_settings(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_save_settings(world: u32, value: String) -> String {
    <Host as PlatformApi>::og_platform_save_settings(&Host, world, value)
}

#[wasm_bindgen]
pub fn og_platform_save_slot(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_save_slot(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_load_slot(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_load_slot(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_products(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_products(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_purchase(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_purchase(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_entitlements(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_entitlements(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_consume(world: u32, purchase: String) -> String {
    <Host as PlatformApi>::og_platform_consume(&Host, world, purchase)
}

#[wasm_bindgen]
pub fn og_platform_grant(world: u32, purchase: String) -> String {
    <Host as PlatformApi>::og_platform_grant(&Host, world, purchase)
}

#[wasm_bindgen]
pub fn og_platform_granted(world: u32, purchase: String) -> bool {
    <Host as PlatformApi>::og_platform_granted(&Host, world, purchase)
}

#[wasm_bindgen]
pub fn og_platform_ad_ready(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_ad_ready(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_show_ad(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_show_ad(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_achievement_unlock(world: u32, id: String) -> String {
    <Host as PlatformApi>::og_platform_achievement_unlock(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_platform_achievement_progress(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_achievement_progress(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_achievements(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_achievements(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_player(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_player(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_friends(world: u32, desc: Option<String>) -> u32 {
    <Host as PlatformApi>::og_platform_friends(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_invite_send(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_invite_send(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_invites(world: u32) -> u32 {
    <Host as PlatformApi>::og_platform_invites(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_share(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_share(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_supports(world: u32, method: String) -> bool {
    <Host as PlatformApi>::og_platform_supports(&Host, world, method)
}

#[wasm_bindgen]
pub fn og_platform_notice(world: u32, notice: u32) -> String {
    <Host as PlatformApi>::og_platform_notice(&Host, world, notice)
}

#[wasm_bindgen]
pub fn og_platform_result(world: u32, request: u32) -> String {
    <Host as PlatformApi>::og_platform_result(&Host, world, request)
}

#[wasm_bindgen]
pub fn og_platform_info(world: u32) -> String {
    <Host as PlatformApi>::og_platform_info(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_journal(world: u32) -> String {
    <Host as PlatformApi>::og_platform_journal(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_now(world: u32) -> f64 {
    <Host as PlatformApi>::og_platform_now(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_room_create(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_room_create(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_room_join(world: u32, room: String) -> String {
    <Host as PlatformApi>::og_platform_room_join(&Host, world, room)
}

#[wasm_bindgen]
pub fn og_platform_quick_play(world: u32, desc: Option<String>) -> String {
    <Host as PlatformApi>::og_platform_quick_play(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_platform_room_leave(world: u32) -> String {
    <Host as PlatformApi>::og_platform_room_leave(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_room_send(world: u32, data: String, to: Option<u32>) -> String {
    <Host as PlatformApi>::og_platform_room_send(&Host, world, data, to)
}

#[wasm_bindgen]
pub fn og_platform_room_peers(world: u32) -> String {
    <Host as PlatformApi>::og_platform_room_peers(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_room_info(world: u32) -> String {
    <Host as PlatformApi>::og_platform_room_info(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_room_message(world: u32, message: u32) -> String {
    <Host as PlatformApi>::og_platform_room_message(&Host, world, message)
}

#[wasm_bindgen]
pub fn og_i18n_load(world: u32, path: String) -> String {
    <Host as I18nApi>::og_i18n_load(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_i18n_set_locale(world: u32, locale: String) -> String {
    <Host as I18nApi>::og_i18n_set_locale(&Host, world, locale)
}

#[wasm_bindgen]
pub fn og_i18n_locale(world: u32) -> String {
    <Host as I18nApi>::og_i18n_locale(&Host, world)
}

#[wasm_bindgen]
pub fn og_i18n_t(world: u32, key: String, args: Option<String>) -> String {
    <Host as I18nApi>::og_i18n_t(&Host, world, key, args)
}

#[wasm_bindgen]
pub fn og_i18n_has(world: u32, key: String) -> bool {
    <Host as I18nApi>::og_i18n_has(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_i18n_format_number(world: u32, value: f64, format: Option<String>) -> String {
    <Host as I18nApi>::og_i18n_format_number(&Host, world, value, format)
}

#[wasm_bindgen]
pub fn og_i18n_format_compact(world: u32, value: f64, decimals: Option<u32>) -> String {
    <Host as I18nApi>::og_i18n_format_compact(&Host, world, value, decimals)
}

#[wasm_bindgen]
pub fn og_i18n_format_currency(world: u32, amount: f64, currency: String) -> String {
    <Host as I18nApi>::og_i18n_format_currency(&Host, world, amount, currency)
}

#[wasm_bindgen]
pub fn og_i18n_format_date(world: u32, time_ms: f64, format: Option<String>) -> String {
    <Host as I18nApi>::og_i18n_format_date(&Host, world, time_ms, format)
}

#[wasm_bindgen]
pub fn og_i18n_format_time(world: u32, time_ms: f64, format: Option<String>) -> String {
    <Host as I18nApi>::og_i18n_format_time(&Host, world, time_ms, format)
}

#[wasm_bindgen]
pub fn og_i18n_format_duration(world: u32, seconds: f64, style: Option<String>) -> String {
    <Host as I18nApi>::og_i18n_format_duration(&Host, world, seconds, style)
}

#[wasm_bindgen]
pub fn og_ui_configure(world: u32, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_configure(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_ui_viewport(world: u32) -> String {
    <Host as UiApi>::og_ui_viewport(&Host, world)
}

#[wasm_bindgen]
pub fn og_ui_theme(world: u32) -> String {
    <Host as UiApi>::og_ui_theme(&Host, world)
}

#[wasm_bindgen]
pub fn og_ui_frame_pointer() -> String {
    <Host as UiApi>::og_ui_frame_pointer(&Host)
}

#[wasm_bindgen]
pub fn og_ui_frame_cursor() -> String {
    <Host as UiApi>::og_ui_frame_cursor(&Host)
}

#[wasm_bindgen]
pub fn og_ui_frame_viewport() -> String {
    <Host as UiApi>::og_ui_frame_viewport(&Host)
}

#[wasm_bindgen]
pub fn og_ui_has_icon(name: String) -> bool {
    <Host as UiApi>::og_ui_has_icon(&Host, name)
}

#[wasm_bindgen]
pub fn og_ui_seconds() -> f64 {
    <Host as UiApi>::og_ui_seconds(&Host)
}

#[wasm_bindgen]
pub fn og_ui_frame_direction() -> String {
    <Host as UiApi>::og_ui_frame_direction(&Host)
}

#[wasm_bindgen]
pub fn og_ui_frame_theme() -> String {
    <Host as UiApi>::og_ui_frame_theme(&Host)
}

#[wasm_bindgen]
pub fn og_ui_set_virtual_cursor(desc: String) {
    <Host as UiApi>::og_ui_set_virtual_cursor(&Host, desc)
}

#[wasm_bindgen]
pub fn og_ui_virtual_cursor() -> String {
    <Host as UiApi>::og_ui_virtual_cursor(&Host)
}

#[wasm_bindgen]
pub fn og_ui_feedback(event: String, rise: f64) {
    <Host as UiApi>::og_ui_feedback(&Host, event, rise)
}

#[wasm_bindgen]
pub fn og_ui_set_haptics(intensity: f64) {
    <Host as UiApi>::og_ui_set_haptics(&Host, intensity)
}

#[wasm_bindgen]
pub fn og_ui_tick() -> u32 {
    <Host as UiApi>::og_ui_tick(&Host)
}

#[wasm_bindgen]
pub fn og_ui_preview_viewport(world: u32, viewport: String) {
    <Host as UiApi>::og_ui_preview_viewport(&Host, world, viewport)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_commands_register(name: String, handler: u32, desc: Option<String>) -> String {
    <Host as CommandsApi>::og_commands_register(&Host, name, handler, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_commands_parse(script: String) -> String {
    <Host as CommandsApi>::og_commands_parse(&Host, script)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_commands_help(world: u32) -> String {
    <Host as CommandsApi>::og_commands_help(&Host, world)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_commands_schedule(world: u32, script: String) -> String {
    <Host as CommandsApi>::og_commands_schedule(&Host, world, script)
}

#[wasm_bindgen]
pub fn og_runtime_probe_get_version(runtime_probe: u32) -> u32 {
    <Host as ProbeApi>::og_runtime_probe_get_version(&Host, runtime_probe)
}

#[wasm_bindgen]
pub fn og_runtime_probe_tick(runtime_probe: u32) -> u32 {
    <Host as ProbeApi>::og_runtime_probe_tick(&Host, runtime_probe)
}

#[wasm_bindgen]
pub fn og_runtime_probe_state(runtime_probe: u32) -> String {
    <Host as ProbeApi>::og_runtime_probe_state(&Host, runtime_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_advance(runtime_probe: u32, steps: u32) -> u32 {
    <Host as ProbeApi>::og_runtime_probe_advance(&Host, runtime_probe, steps)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_pause(runtime_probe: u32) -> bool {
    <Host as ProbeApi>::og_runtime_probe_pause(&Host, runtime_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_resume(runtime_probe: u32) -> bool {
    <Host as ProbeApi>::og_runtime_probe_resume(&Host, runtime_probe)
}

#[wasm_bindgen]
pub fn og_runtime_probe_input(runtime_probe: u32, event: String) -> bool {
    <Host as ProbeApi>::og_runtime_probe_input(&Host, runtime_probe, event)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_accepted(runtime_probe: u32) -> u32 {
    <Host as ProbeApi>::og_runtime_probe_accepted(&Host, runtime_probe)
}

#[wasm_bindgen]
pub fn og_runtime_probe_pick(runtime_probe: u32, x: f64, y: f64) -> String {
    <Host as ProbeApi>::og_runtime_probe_pick(&Host, runtime_probe, x, y)
}

#[wasm_bindgen]
pub fn og_runtime_probe_entity(runtime_probe: u32, id: String) -> String {
    <Host as ProbeApi>::og_runtime_probe_entity(&Host, runtime_probe, id)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_bounds(runtime_probe: u32, id: String) -> String {
    <Host as ProbeApi>::og_runtime_probe_bounds(&Host, runtime_probe, id)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_patch(runtime_probe: u32, id: String, parameter: String, value: String) -> bool {
    <Host as ProbeApi>::og_runtime_probe_patch(&Host, runtime_probe, id, parameter, value)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_runtime_probe_run(runtime_probe: u32, program: String) -> String {
    <Host as ProbeApi>::og_runtime_probe_run(&Host, runtime_probe, program)
}

#[wasm_bindgen]
pub fn og_engine_probe_hello(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_hello(&Host, engine_probe)
}

#[wasm_bindgen]
pub fn og_engine_probe_capture(engine_probe: u32, width: Option<u32>, height: Option<u32>, fit: Option<u32>, region: Option<String>) -> String {
    <Host as ProbeApi>::og_engine_probe_capture(&Host, engine_probe, width, height, fit, region)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_capture_views(engine_probe: u32, views: String, width: Option<u32>, height: Option<u32>, fit: Option<u32>) -> String {
    <Host as ProbeApi>::og_engine_probe_capture_views(&Host, engine_probe, views, width, height, fit)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_visible(engine_probe: u32, id: String) -> String {
    <Host as ProbeApi>::og_engine_probe_visible(&Host, engine_probe, id)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_marks(engine_probe: u32, desc: Option<String>) -> String {
    <Host as ProbeApi>::og_engine_probe_marks(&Host, engine_probe, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_timeline(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_timeline(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_sweep(engine_probe: u32, desc: Option<String>) -> String {
    <Host as ProbeApi>::og_engine_probe_sweep(&Host, engine_probe, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_reload(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_reload(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_snapshot(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_snapshot(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_restore(engine_probe: u32, id: String) -> bool {
    <Host as ProbeApi>::og_engine_probe_restore(&Host, engine_probe, id)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_dump(engine_probe: u32, desc: Option<String>) -> String {
    <Host as ProbeApi>::og_engine_probe_dump(&Host, engine_probe, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_commands(engine_probe: u32, script: String) -> String {
    <Host as ProbeApi>::og_engine_probe_commands(&Host, engine_probe, script)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_listen(engine_probe: u32, seconds: Option<f64>) -> String {
    <Host as ProbeApi>::og_engine_probe_listen(&Host, engine_probe, seconds)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_memory(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_memory(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_describe(engine_probe: u32, joints: Option<String>) -> String {
    <Host as ProbeApi>::og_engine_probe_describe(&Host, engine_probe, joints)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_expect(engine_probe: u32, what: String) -> String {
    <Host as ProbeApi>::og_engine_probe_expect(&Host, engine_probe, what)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_expectations(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_expectations(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_assertions(engine_probe: u32, take: Option<bool>) -> String {
    <Host as ProbeApi>::og_engine_probe_assertions(&Host, engine_probe, take)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_assertion_events(engine_probe: u32) -> String {
    <Host as ProbeApi>::og_engine_probe_assertion_events(&Host, engine_probe)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_observe(engine_probe: u32, records: Option<String>, types: Option<bool>) -> String {
    <Host as ProbeApi>::og_engine_probe_observe(&Host, engine_probe, records, types)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_save_scene(engine_probe: u32, write: Option<bool>, spawned: Option<bool>) -> String {
    <Host as ProbeApi>::og_engine_probe_save_scene(&Host, engine_probe, write, spawned)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_pick_ui(engine_probe: u32, x: f64, y: f64) -> String {
    <Host as ProbeApi>::og_engine_probe_pick_ui(&Host, engine_probe, x, y)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_logs(engine_probe: u32, take: Option<bool>) -> String {
    <Host as ProbeApi>::og_engine_probe_logs(&Host, engine_probe, take)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_set(engine_probe: u32, target: String, value: String) -> String {
    <Host as ProbeApi>::og_engine_probe_set(&Host, engine_probe, target, value)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_get(engine_probe: u32, target: String) -> String {
    <Host as ProbeApi>::og_engine_probe_get(&Host, engine_probe, target)
}

#[wasm_bindgen]
pub fn og_terrain_load(world: u32, path: String) -> String {
    <Host as TerrainApi>::og_terrain_load(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_terrain_create(world: u32, desc: Option<Vec<u8>>) -> String {
    <Host as TerrainApi>::og_terrain_create(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_terrain_sample(world: u32, desc: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_sample(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_terrain_bounds(world: u32, data: f64, origin: Option<String>, quaternion: Option<String>, scale: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_bounds(&Host, world, data, origin, quaternion, scale)
}

#[wasm_bindgen]
pub fn og_terrain_edit(world: u32, data: f64, brush: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_edit(&Host, world, data, brush)
}

#[wasm_bindgen]
pub fn og_terrain_stamp(world: u32, desc: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_stamp(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_terrain_unstamp(world: u32, data: f64, stamp: u32) -> String {
    <Host as TerrainApi>::og_terrain_unstamp(&Host, world, data, stamp)
}

#[wasm_bindgen]
pub fn og_terrain_layer(world: u32, data: f64, index: u32) -> String {
    <Host as TerrainApi>::og_terrain_layer(&Host, world, data, index)
}

#[wasm_bindgen]
pub fn og_terrain_set_layer(world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_set_layer(&Host, world, data, index, layer)
}

#[wasm_bindgen]
pub fn og_stream_status(world: u32, group: String) -> String {
    <Host as StreamApi>::og_stream_status(&Host, world, group)
}

#[wasm_bindgen]
pub fn og_stream_request(world: u32, group: String) -> String {
    <Host as StreamApi>::og_stream_request(&Host, world, group)
}

#[wasm_bindgen]
pub fn og_stream_release(world: u32, group: String) -> String {
    <Host as StreamApi>::og_stream_release(&Host, world, group)
}

#[wasm_bindgen]
pub fn og_stream_budget(world: u32) -> String {
    <Host as StreamApi>::og_stream_budget(&Host, world)
}

#[wasm_bindgen]
pub fn og_stream_set_budget(world: u32, bytes: f64) -> String {
    <Host as StreamApi>::og_stream_set_budget(&Host, world, bytes)
}

#[wasm_bindgen]
pub fn og_video_capabilities(world: u32) -> String {
    <Host as VideoApi>::og_video_capabilities(&Host, world)
}

#[wasm_bindgen]
pub fn og_video_set_quality(world: u32, quality: String) -> String {
    <Host as VideoApi>::og_video_set_quality(&Host, world, quality)
}

#[wasm_bindgen]
pub fn og_video_buffering(world: u32) -> String {
    <Host as VideoApi>::og_video_buffering(&Host, world)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_line(world: u32, from: String, to: String, desc: Option<String>) {
    <Host as DebugApi>::og_debug_line(&Host, world, from, to, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_box(world: u32, center: String, size: String, desc: Option<String>) {
    <Host as DebugApi>::og_debug_box(&Host, world, center, size, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_sphere(world: u32, center: String, radius: f64, desc: Option<String>) {
    <Host as DebugApi>::og_debug_sphere(&Host, world, center, radius, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_arrow(world: u32, from: String, to: String, desc: Option<String>) {
    <Host as DebugApi>::og_debug_arrow(&Host, world, from, to, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_axes(world: u32, position: String, size: f64, desc: Option<String>) {
    <Host as DebugApi>::og_debug_axes(&Host, world, position, size, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_text(world: u32, position: String, text: String, desc: Option<String>) {
    <Host as DebugApi>::og_debug_text(&Host, world, position, text, desc)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_clear(world: u32) {
    <Host as DebugApi>::og_debug_clear(&Host, world)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_set_view(world: u32, view: String) {
    <Host as DebugApi>::og_debug_set_view(&Host, world, view)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_view(world: u32) -> String {
    <Host as DebugApi>::og_debug_view(&Host, world)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_set_enabled(world: u32, enabled: bool) {
    <Host as DebugApi>::og_debug_set_enabled(&Host, world, enabled)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_debug_enabled(world: u32) -> bool {
    <Host as DebugApi>::og_debug_enabled(&Host, world)
}

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_assert_record(world: u32, record: String) -> u32 {
    <Host as AssertApi>::og_assert_record(&Host, world, record)
}

#[wasm_bindgen]
pub fn og_path_create(world: u32, desc: Option<String>) -> String {
    <Host as PathApi>::og_path_create(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_path_find(world: u32, name: String) -> String {
    <Host as PathApi>::og_path_find(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_path_sample(world: u32, path: f64, t: f64) -> String {
    <Host as PathApi>::og_path_sample(&Host, world, path, t)
}

#[wasm_bindgen]
pub fn og_path_length(world: u32, path: f64) -> String {
    <Host as PathApi>::og_path_length(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_path_nearest(world: u32, path: f64, point: String) -> String {
    <Host as PathApi>::og_path_nearest(&Host, world, path, point)
}

#[wasm_bindgen]
pub fn og_timeline_captions(world: u32) -> String {
    <Host as TimelineApi>::og_timeline_captions(&Host, world)
}

#[wasm_bindgen]
pub fn og_timeline_info(world: u32, asset: f64) -> String {
    <Host as TimelineApi>::og_timeline_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_timeline_play(world: u32, entity: f64) -> String {
    <Host as TimelineApi>::og_timeline_play(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_timeline_pause(world: u32, entity: f64) -> String {
    <Host as TimelineApi>::og_timeline_pause(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_timeline_seek(world: u32, entity: f64, time: f64) -> String {
    <Host as TimelineApi>::og_timeline_seek(&Host, world, entity, time)
}

#[wasm_bindgen]
pub fn og_timeline_stop(world: u32, entity: f64) -> String {
    <Host as TimelineApi>::og_timeline_stop(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_net_submit(world: u32, input: Option<String>) -> String {
    <Host as NetApi>::og_net_submit(&Host, world, input)
}

#[wasm_bindgen]
pub fn og_net_shape(world: u32, desc: Option<String>) -> String {
    <Host as NetApi>::og_net_shape(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_net_submit_input(world: u32, component: String, value: String) -> String {
    <Host as NetApi>::og_net_submit_input(&Host, world, component, value)
}

#[wasm_bindgen]
pub fn og_net_command(world: u32, component: String, value: String) -> String {
    <Host as NetApi>::og_net_command(&Host, world, component, value)
}

#[wasm_bindgen]
pub fn og_net_commands(world: u32, component: String) -> String {
    <Host as NetApi>::og_net_commands(&Host, world, component)
}

#[wasm_bindgen]
pub fn og_net_status(world: u32) -> String {
    <Host as NetApi>::og_net_status(&Host, world)
}

#[wasm_bindgen]
pub fn og_net_resimulating(world: u32) -> bool {
    <Host as NetApi>::og_net_resimulating(&Host, world)
}

#[wasm_bindgen]
pub fn og_net_final_hash(world: u32, tick: f64) -> String {
    <Host as NetApi>::og_net_final_hash(&Host, world, tick)
}

#[wasm_bindgen]
pub fn og_net_hash(world: u32) -> String {
    <Host as NetApi>::og_net_hash(&Host, world)
}

#[wasm_bindgen]
pub fn og_shared_shape(world: u32, desc: Option<String>) -> String {
    <Host as SharedApi>::og_shared_shape(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_shared_join(world: u32, desc: Option<String>) -> String {
    <Host as SharedApi>::og_shared_join(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_shared_leave(world: u32) -> String {
    <Host as SharedApi>::og_shared_leave(&Host, world)
}

#[wasm_bindgen]
pub fn og_shared_teleport(world: u32, position: String) -> String {
    <Host as SharedApi>::og_shared_teleport(&Host, world, position)
}

#[wasm_bindgen]
pub fn og_shared_send(world: u32, component: String, value: String) -> String {
    <Host as SharedApi>::og_shared_send(&Host, world, component, value)
}

#[wasm_bindgen]
pub fn og_shared_events(world: u32, component: String) -> String {
    <Host as SharedApi>::og_shared_events(&Host, world, component)
}

#[wasm_bindgen]
pub fn og_shared_put(world: u32, key: String, component: String, value: String, desc: Option<String>) -> String {
    <Host as SharedApi>::og_shared_put(&Host, world, key, component, value, desc)
}

#[wasm_bindgen]
pub fn og_shared_remove(world: u32, key: String, expect: String) -> String {
    <Host as SharedApi>::og_shared_remove(&Host, world, key, expect)
}

#[wasm_bindgen]
pub fn og_shared_object(world: u32, entity: f64) -> String {
    <Host as SharedApi>::og_shared_object(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_shared_remote(world: u32, entity: f64) -> String {
    <Host as SharedApi>::og_shared_remote(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_shared_status(world: u32) -> String {
    <Host as SharedApi>::og_shared_status(&Host, world)
}

#[wasm_bindgen]
pub fn og_motion_pose(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_pose(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_place(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_place(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_face(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_face(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_walk_to(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_walk_to(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_jump(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_jump(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_clear(world: u32, entity: f64) -> String {
    <Host as MotionApi>::og_motion_clear(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_motion_look_at(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_look_at(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_react(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as MotionApi>::og_motion_react(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_motion_status(world: u32, entity: f64) -> String {
    <Host as MotionApi>::og_motion_status(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_motion_capabilities(world: u32) -> String {
    <Host as MotionApi>::og_motion_capabilities(&Host, world)
}

#[wasm_bindgen]
pub fn og_motion_set_backend(world: u32, backend: String) -> String {
    <Host as MotionApi>::og_motion_set_backend(&Host, world, backend)
}

#[wasm_bindgen]
pub fn og_motion_presentation(world: u32, entity: f64) -> String {
    <Host as MotionApi>::og_motion_presentation(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_motion_poses() -> String {
    <Host as MotionApi>::og_motion_poses(&Host)
}

#[wasm_bindgen]
pub fn og_destruction_damage(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_damage(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_destruction_explode(world: u32, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_explode(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_destruction_fracture(world: u32, entity: f64) -> String {
    <Host as DestructionApi>::og_destruction_fracture(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_destruction_info(world: u32, entity: f64) -> String {
    <Host as DestructionApi>::og_destruction_info(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_destruction_material(world: u32, kind: u32) -> String {
    <Host as DestructionApi>::og_destruction_material(&Host, world, kind)
}

#[wasm_bindgen]
pub fn og_destruction_capabilities(world: u32) -> String {
    <Host as DestructionApi>::og_destruction_capabilities(&Host, world)
}

#[wasm_bindgen]
pub fn og_destruction_carve2d(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_carve2d(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_destruction_fill2d(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_fill2d(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_destruction_explode2d(world: u32, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_explode2d(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_destruction_fracture2d(world: u32, entity: f64, desc: Option<String>) -> String {
    <Host as DestructionApi>::og_destruction_fracture2d(&Host, world, entity, desc)
}

#[wasm_bindgen]
pub fn og_destruction_cell2d(world: u32, entity: f64, point: String) -> String {
    <Host as DestructionApi>::og_destruction_cell2d(&Host, world, entity, point)
}

#[wasm_bindgen]
pub fn og_vfx_spawn(world: u32, desc: Option<String>) -> String {
    <Host as VfxApi>::og_vfx_spawn(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_vfx_info(world: u32, asset: f64) -> String {
    <Host as VfxApi>::og_vfx_info(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_sight_visible(world: u32, team: u32, point: String) -> String {
    <Host as SightApi>::og_sight_visible(&Host, world, team, point)
}

#[wasm_bindgen]
pub fn og_sight_explored(world: u32, team: u32, point: String) -> String {
    <Host as SightApi>::og_sight_explored(&Host, world, team, point)
}

#[wasm_bindgen]
pub fn og_sight_reveal(world: u32, team: u32, point: String, radius: f64) -> String {
    <Host as SightApi>::og_sight_reveal(&Host, world, team, point, radius)
}

#[wasm_bindgen]
pub fn og_sight_forget(world: u32, team: u32) -> String {
    <Host as SightApi>::og_sight_forget(&Host, world, team)
}

#[wasm_bindgen]
pub fn og_flow_load(world: u32, dialogue: String) -> String {
    <Host as FlowApi>::og_flow_load(&Host, world, dialogue)
}

#[wasm_bindgen]
pub fn og_flow_start(world: u32, desc: Option<String>) -> String {
    <Host as FlowApi>::og_flow_start(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_flow_current(world: u32) -> String {
    <Host as FlowApi>::og_flow_current(&Host, world)
}

#[wasm_bindgen]
pub fn og_flow_view(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_view(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_choose(world: u32, flow: f64, index: u32) -> String {
    <Host as FlowApi>::og_flow_choose(&Host, world, flow, index)
}

#[wasm_bindgen]
pub fn og_flow_advance(world: u32, flow: f64, outcome: Option<String>) -> String {
    <Host as FlowApi>::og_flow_advance(&Host, world, flow, outcome)
}

#[wasm_bindgen]
pub fn og_flow_play(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_play(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_pause(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_pause(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_set_speed(world: u32, flow: f64, speed: f64) -> String {
    <Host as FlowApi>::og_flow_set_speed(&Host, world, flow, speed)
}

#[wasm_bindgen]
pub fn og_flow_seek(world: u32, flow: f64, time: f64) -> String {
    <Host as FlowApi>::og_flow_seek(&Host, world, flow, time)
}

#[wasm_bindgen]
pub fn og_flow_inspect(world: u32, flow: f64, hotspot: String) -> String {
    <Host as FlowApi>::og_flow_inspect(&Host, world, flow, hotspot)
}

#[wasm_bindgen]
pub fn og_flow_gesture(world: u32, flow: f64, desc: Option<String>) -> String {
    <Host as FlowApi>::og_flow_gesture(&Host, world, flow, desc)
}

#[wasm_bindgen]
pub fn og_flow_set_music(world: u32, flow: f64, desc: Option<String>) -> String {
    <Host as FlowApi>::og_flow_set_music(&Host, world, flow, desc)
}

#[wasm_bindgen]
pub fn og_flow_history(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_history(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_checkpoint(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_checkpoint(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_jump(world: u32, flow: f64, desc: Option<String>) -> String {
    <Host as FlowApi>::og_flow_jump(&Host, world, flow, desc)
}

#[wasm_bindgen]
pub fn og_flow_skip(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_skip(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_flow_chart(world: u32, dialogue: String) -> String {
    <Host as FlowApi>::og_flow_chart(&Host, world, dialogue)
}

#[wasm_bindgen]
pub fn og_flow_stop(world: u32, flow: f64) -> String {
    <Host as FlowApi>::og_flow_stop(&Host, world, flow)
}

#[wasm_bindgen]
pub fn og_scene_preload(world: u32, scene: String) -> String {
    <Host as SceneApi>::og_scene_preload(&Host, world, scene)
}

#[wasm_bindgen]
pub fn og_scene_progress(world: u32, scene: String) -> String {
    <Host as SceneApi>::og_scene_progress(&Host, world, scene)
}

#[wasm_bindgen]
pub fn og_scene_release(world: u32, scene: String) -> String {
    <Host as SceneApi>::og_scene_release(&Host, world, scene)
}

#[wasm_bindgen]
pub fn og_scene_change(world: u32, scene: String) -> String {
    <Host as SceneApi>::og_scene_change(&Host, world, scene)
}

#[wasm_bindgen]
pub fn og_scene_current(world: u32) -> String {
    <Host as SceneApi>::og_scene_current(&Host, world)
}

#[wasm_bindgen]
pub fn og_loading_stage(world: u32, desc: Option<String>) -> String {
    <Host as LoadingApi>::og_loading_stage(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_loading_done(world: u32) -> String {
    <Host as LoadingApi>::og_loading_done(&Host, world)
}

#[wasm_bindgen]
pub fn og_loading_idle(world: u32) -> bool {
    <Host as LoadingApi>::og_loading_idle(&Host, world)
}

#[wasm_bindgen]
pub fn og_loading_status(world: u32) -> String {
    <Host as LoadingApi>::og_loading_status(&Host, world)
}

#[wasm_bindgen]
pub fn og_play_candidates(world: u32, desc: Option<String>) -> String {
    <Host as PlayApi>::og_play_candidates(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_table_load(world: u32, path: String, schema: String) -> String {
    <Host as TableApi>::og_table_load(&Host, world, path, schema)
}

#[wasm_bindgen]
pub fn og_table_parse(world: u32, path: String, text: String, schema: String) -> String {
    <Host as TableApi>::og_table_parse(&Host, world, path, text, schema)
}

#[wasm_bindgen]
pub fn og_table_row(world: u32, table: String, id: String) -> String {
    <Host as TableApi>::og_table_row(&Host, world, table, id)
}

#[wasm_bindgen]
pub fn og_table_rows(world: u32, table: String) -> String {
    <Host as TableApi>::og_table_rows(&Host, world, table)
}

#[wasm_bindgen]
pub fn og_table_get(world: u32, row: f64, schema: String) -> String {
    <Host as TableApi>::og_table_get(&Host, world, row, schema)
}

#[wasm_bindgen]
pub fn og_table_name(world: u32, row: f64) -> String {
    <Host as TableApi>::og_table_name(&Host, world, row)
}

#[wasm_bindgen]
pub fn og_table_missing(world: u32) -> String {
    <Host as TableApi>::og_table_missing(&Host, world)
}
