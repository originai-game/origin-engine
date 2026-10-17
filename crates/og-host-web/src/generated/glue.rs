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
pub fn og_engine_write_resource(world: u32, resource: String, value: String) -> String {
    <Host as Lifecycle>::og_engine_write_resource(&Host, world, resource, value)
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
pub fn og_world_find(world: u32, name: String) -> String {
    <Host as WorldApi>::og_world_find(&Host, world, name)
}

#[wasm_bindgen]
pub fn og_world_name_of(world: u32, entity: f64) -> String {
    <Host as WorldApi>::og_world_name_of(&Host, world, entity)
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
pub fn og_query_archetypes(query: u32) -> String {
    <Host as WorldApi>::og_query_archetypes(&Host, query)
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
pub fn og_archetype_view_entity(archetype_view: u32, index: u32) -> f64 {
    <Host as WorldApi>::og_archetype_view_entity(&Host, archetype_view, index)
}

#[wasm_bindgen]
pub fn og_archetype_view_has(archetype_view: u32, component: String) -> bool {
    <Host as WorldApi>::og_archetype_view_has(&Host, archetype_view, component)
}

#[wasm_bindgen]
pub fn og_archetype_view_column(archetype_view: u32, component: String) -> Vec<u32> {
    <Host as WorldApi>::og_archetype_view_column(&Host, archetype_view, component)
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
pub fn og_transform_look_at(eye: String, target: String, up: Option<String>) -> String {
    <Host as TransformApi>::og_transform_look_at(&Host, eye, target, up)
}

#[wasm_bindgen]
pub fn og_transform_from_axis_angle(axis: String, angle: f64) -> String {
    <Host as TransformApi>::og_transform_from_axis_angle(&Host, axis, angle)
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
pub fn og_rng_fork(world: u32, label: String) -> u32 {
    <Host as RngApi>::og_rng_fork(&Host, world, label)
}

#[wasm_bindgen]
pub fn og_input_down(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_down(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_pressed(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_pressed(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_released(world: u32, key: String) -> bool {
    <Host as InputApi>::og_input_released(&Host, world, key)
}

#[wasm_bindgen]
pub fn og_input_axis(world: u32, negative: String, positive: String) -> f64 {
    <Host as InputApi>::og_input_axis(&Host, world, negative, positive)
}

#[wasm_bindgen]
pub fn og_input_button_down(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_down(&Host, world, button)
}

#[wasm_bindgen]
pub fn og_input_button_pressed(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_pressed(&Host, world, button)
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
pub fn og_render_standard_material(world: u32, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_standard_material(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_render_update_material(world: u32, material: f64, desc: Option<String>) -> String {
    <Host as RenderApi>::og_render_update_material(&Host, world, material, desc)
}

#[wasm_bindgen]
pub fn og_render_capabilities(world: u32) -> String {
    <Host as RenderApi>::og_render_capabilities(&Host, world)
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
pub fn og_load_status(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_status(&Host, world, asset)
}

#[wasm_bindgen]
pub fn og_load_gltf_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_gltf_info(&Host, world, asset)
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
pub fn og_engine_probe_capture(engine_probe: u32, width: Option<u32>, height: Option<u32>) -> String {
    <Host as ProbeApi>::og_engine_probe_capture(&Host, engine_probe, width, height)
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
