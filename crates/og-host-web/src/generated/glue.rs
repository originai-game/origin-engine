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
pub fn og_time_set_scale(world: u32, scale: f64) -> String {
    <Host as TimeApi>::og_time_set_scale(&Host, world, scale)
}

#[wasm_bindgen]
pub fn og_time_hit_stop(world: u32, ticks: u32) -> String {
    <Host as TimeApi>::og_time_hit_stop(&Host, world, ticks)
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
pub fn og_input_axis2(world: u32, left: String, right: String, down: String, up: String) -> String {
    <Host as InputApi>::og_input_axis2(&Host, world, left, right, down, up)
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
pub fn og_input_button_released(world: u32, button: String) -> bool {
    <Host as InputApi>::og_input_button_released(&Host, world, button)
}

#[wasm_bindgen]
pub fn og_pointers_down(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_down(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_pressed(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_pressed(&Host, world, id)
}

#[wasm_bindgen]
pub fn og_pointers_released(world: u32, id: u32) -> bool {
    <Host as InputApi>::og_pointers_released(&Host, world, id)
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
pub fn og_gamepad_pressed(world: u32, button: String, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_pressed(&Host, world, button, pad)
}

#[wasm_bindgen]
pub fn og_gamepad_released(world: u32, button: String, pad: Option<u32>) -> bool {
    <Host as InputApi>::og_gamepad_released(&Host, world, button, pad)
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
pub fn og_render_set_shader_param(world: u32, material: f64, name: String, value: String) -> String {
    <Host as RenderApi>::og_render_set_shader_param(&Host, world, material, name, value)
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
pub fn og_load_bundle(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_bundle(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_bundle_info(world: u32, asset: f64) -> String {
    <Host as LoadApi>::og_load_bundle_info(&Host, world, asset)
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
pub fn og_load_emitter(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_emitter(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_shader(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_shader(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_texture(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_texture(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_skin(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_skin(&Host, world, path)
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
pub fn og_load_flipbook(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_flipbook(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_load_tilemap(world: u32, path: String) -> String {
    <Host as LoadApi>::og_load_tilemap(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_physics_raycast(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_raycast(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_shape_cast(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_shape_cast(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_physics_overlap(world: u32, desc: Option<String>) -> String {
    <Host as PhysicsApi>::og_physics_overlap(&Host, world, desc)
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
pub fn og_physics_cloth_points(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_cloth_points(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_fluid_particles(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_fluid_particles(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_water_height(world: u32, entity: f64, x: f64, z: f64) -> String {
    <Host as PhysicsApi>::og_physics_water_height(&Host, world, entity, x, z)
}

#[wasm_bindgen]
pub fn og_physics_vehicle_wheels(world: u32, entity: f64) -> String {
    <Host as PhysicsApi>::og_physics_vehicle_wheels(&Host, world, entity)
}

#[wasm_bindgen]
pub fn og_physics_ragdoll_rig_from_skeleton(world: u32, skeleton: f64) -> String {
    <Host as PhysicsApi>::og_physics_ragdoll_rig_from_skeleton(&Host, world, skeleton)
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
pub fn og_atmosphere_time_of_day(world: u32) -> f64 {
    <Host as AtmosphereApi>::og_atmosphere_time_of_day(&Host, world)
}

#[wasm_bindgen]
pub fn og_anim_graph(world: u32, desc: Option<String>) -> String {
    <Host as AnimApi>::og_anim_graph(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_tween_to(world: u32, desc: Option<String>) -> String {
    <Host as TweenApi>::og_tween_to(&Host, world, desc)
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
pub fn og_audio_output_latency(world: u32) -> f64 {
    <Host as AudioApi>::og_audio_output_latency(&Host, world)
}

#[wasm_bindgen]
pub fn og_nav_grid(world: u32, desc: Option<String>) -> String {
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
pub fn og_nav_load(world: u32, path: String) -> String {
    <Host as NavApi>::og_nav_load(&Host, world, path)
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
pub fn og_ai_noise(world: u32, desc: Option<String>) -> String {
    <Host as AiApi>::og_ai_noise(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_ai_describe(world: u32, entity: f64) -> String {
    <Host as AiApi>::og_ai_describe(&Host, world, entity)
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
pub fn og_platform_result(world: u32, request: u32) -> String {
    <Host as PlatformApi>::og_platform_result(&Host, world, request)
}

#[wasm_bindgen]
pub fn og_platform_info(world: u32) -> String {
    <Host as PlatformApi>::og_platform_info(&Host, world)
}

#[wasm_bindgen]
pub fn og_platform_now(world: u32) -> f64 {
    <Host as PlatformApi>::og_platform_now(&Host, world)
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
pub fn og_ui_open(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_open(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_open_panel(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_open_panel(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_open_scroll(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_open_scroll(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_close() -> String {
    <Host as UiApi>::og_ui_close(&Host)
}

#[wasm_bindgen]
pub fn og_ui_label(id: String, text: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_label(&Host, id, text, desc)
}

#[wasm_bindgen]
pub fn og_ui_button(id: String, text: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_button(&Host, id, text, desc)
}

#[wasm_bindgen]
pub fn og_ui_image(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_image(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_progress(id: String, value: f64, desc: Option<String>) {
    <Host as UiApi>::og_ui_progress(&Host, id, value, desc)
}

#[wasm_bindgen]
pub fn og_ui_slider(id: String, value: f64, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_slider(&Host, id, value, desc)
}

#[wasm_bindgen]
pub fn og_ui_toggle(id: String, on: bool, label: String) -> String {
    <Host as UiApi>::og_ui_toggle(&Host, id, on, label)
}

#[wasm_bindgen]
pub fn og_ui_text_field(id: String, value: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_text_field(&Host, id, value, desc)
}

#[wasm_bindgen]
pub fn og_ui_tooltip(text: String) {
    <Host as UiApi>::og_ui_tooltip(&Host, text)
}

#[wasm_bindgen]
pub fn og_ui_breakpoint() -> String {
    <Host as UiApi>::og_ui_breakpoint(&Host)
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
pub fn og_ui_frame_theme() -> String {
    <Host as UiApi>::og_ui_frame_theme(&Host)
}

#[wasm_bindgen]
pub fn og_ui_open_anchor(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_open_anchor(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_open_transition(id: String, show: bool, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_open_transition(&Host, id, show, desc)
}

#[wasm_bindgen]
pub fn og_ui_open_modal(id: String, open: bool, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_open_modal(&Host, id, open, desc)
}

#[wasm_bindgen]
pub fn og_ui_close_modal() {
    <Host as UiApi>::og_ui_close_modal(&Host)
}

#[wasm_bindgen]
pub fn og_ui_spotlight(id: String, hole: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_spotlight(&Host, id, hole, desc)
}

#[wasm_bindgen]
pub fn og_ui_bounds(path: String) -> String {
    <Host as UiApi>::og_ui_bounds(&Host, path)
}

#[wasm_bindgen]
pub fn og_ui_open_list(id: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_open_list(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_open_grid(id: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_open_grid(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_list_item(index: u32, selected: bool) -> String {
    <Host as UiApi>::og_ui_list_item(&Host, index, selected)
}

#[wasm_bindgen]
pub fn og_ui_grid_item(index: u32, selected: bool) -> String {
    <Host as UiApi>::og_ui_grid_item(&Host, index, selected)
}

#[wasm_bindgen]
pub fn og_ui_close_list() -> String {
    <Host as UiApi>::og_ui_close_list(&Host)
}

#[wasm_bindgen]
pub fn og_ui_dragging(id: String) -> String {
    <Host as UiApi>::og_ui_dragging(&Host, id)
}

#[wasm_bindgen]
pub fn og_ui_tabs(id: String, items: String, selected: u32, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_tabs(&Host, id, items, selected, desc)
}

#[wasm_bindgen]
pub fn og_ui_select(id: String, options: String, selected: u32, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_select(&Host, id, options, selected, desc)
}

#[wasm_bindgen]
pub fn og_ui_stepper(id: String, value: f64, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_stepper(&Host, id, value, desc)
}

#[wasm_bindgen]
pub fn og_ui_radio(id: String, options: String, selected: u32, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_radio(&Host, id, options, selected, desc)
}

#[wasm_bindgen]
pub fn og_ui_checkbox(id: String, on: bool, label: String) -> String {
    <Host as UiApi>::og_ui_checkbox(&Host, id, on, label)
}

#[wasm_bindgen]
pub fn og_ui_checkbox_list(id: String, labels: String, values: String) -> String {
    <Host as UiApi>::og_ui_checkbox_list(&Host, id, labels, values)
}

#[wasm_bindgen]
pub fn og_ui_chip(id: String, text: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_chip(&Host, id, text, desc)
}

#[wasm_bindgen]
pub fn og_ui_badge(id: String, text: String, tone: String) {
    <Host as UiApi>::og_ui_badge(&Host, id, text, tone)
}

#[wasm_bindgen]
pub fn og_ui_divider(id: String, vertical: bool) {
    <Host as UiApi>::og_ui_divider(&Host, id, vertical)
}

#[wasm_bindgen]
pub fn og_ui_spacer(id: String, size: String) {
    <Host as UiApi>::og_ui_spacer(&Host, id, size)
}

#[wasm_bindgen]
pub fn og_ui_key_value(id: String, key: String, value: String, mono: bool) {
    <Host as UiApi>::og_ui_key_value(&Host, id, key, value, mono)
}

#[wasm_bindgen]
pub fn og_ui_skeleton(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_skeleton(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_spinner(id: String, size: f64) {
    <Host as UiApi>::og_ui_spinner(&Host, id, size)
}

#[wasm_bindgen]
pub fn og_ui_icon(id: String, name: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_icon(&Host, id, name, desc)
}

#[wasm_bindgen]
pub fn og_ui_meter(id: String, value: f64, max: f64, desc: Option<String>) {
    <Host as UiApi>::og_ui_meter(&Host, id, value, max, desc)
}

#[wasm_bindgen]
pub fn og_ui_cooldown(id: String, remaining: f64, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_cooldown(&Host, id, remaining, desc)
}

#[wasm_bindgen]
pub fn og_ui_minimap(id: String, desc: Option<String>) {
    <Host as UiApi>::og_ui_minimap(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_dialogue(id: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_dialogue(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_joystick(id: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_joystick(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_dpad(id: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_dpad(&Host, id, desc)
}

#[wasm_bindgen]
pub fn og_ui_action_button(id: String, text: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_action_button(&Host, id, text, desc)
}

#[wasm_bindgen]
pub fn og_ui_toast(world: u32, text: String, desc: Option<String>) -> u32 {
    <Host as UiApi>::og_ui_toast(&Host, world, text, desc)
}

#[wasm_bindgen]
pub fn og_ui_dismiss_toast(world: u32, toast_id: u32) {
    <Host as UiApi>::og_ui_dismiss_toast(&Host, world, toast_id)
}

#[wasm_bindgen]
pub fn og_ui_navigate(world: u32, intent: String) {
    <Host as UiApi>::og_ui_navigate(&Host, world, intent)
}

#[wasm_bindgen]
pub fn og_ui_back() -> bool {
    <Host as UiApi>::og_ui_back(&Host)
}

#[wasm_bindgen]
pub fn og_ui_set_navigation(on: bool) {
    <Host as UiApi>::og_ui_set_navigation(&Host, on)
}

#[wasm_bindgen]
pub fn og_ui_memory(id: String, initial: f64) -> f64 {
    <Host as UiApi>::og_ui_memory(&Host, id, initial)
}

#[wasm_bindgen]
pub fn og_ui_remember(id: String, value: f64) {
    <Host as UiApi>::og_ui_remember(&Host, id, value)
}

#[wasm_bindgen]
pub fn og_ui_tick() -> u32 {
    <Host as UiApi>::og_ui_tick(&Host)
}

#[wasm_bindgen]
pub fn og_ui_counter(id: String, value: f64, desc: Option<String>) -> f64 {
    <Host as UiApi>::og_ui_counter(&Host, id, value, desc)
}

#[wasm_bindgen]
pub fn og_ui_burst(desc: Option<String>) {
    <Host as UiApi>::og_ui_burst(&Host, desc)
}

#[wasm_bindgen]
pub fn og_ui_shake(desc: Option<String>) {
    <Host as UiApi>::og_ui_shake(&Host, desc)
}

#[wasm_bindgen]
pub fn og_ui_screen_transition(id: String, screen: String, desc: Option<String>) -> String {
    <Host as UiApi>::og_ui_screen_transition(&Host, id, screen, desc)
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

#[cfg(not(feature = "publish"))]
#[wasm_bindgen]
pub fn og_engine_probe_listen(engine_probe: u32, seconds: Option<f64>) -> String {
    <Host as ProbeApi>::og_engine_probe_listen(&Host, engine_probe, seconds)
}

#[wasm_bindgen]
pub fn og_terrain_load(world: u32, path: String) -> String {
    <Host as TerrainApi>::og_terrain_load(&Host, world, path)
}

#[wasm_bindgen]
pub fn og_terrain_sample(world: u32, desc: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_sample(&Host, world, desc)
}

#[wasm_bindgen]
pub fn og_terrain_bounds(world: u32, data: f64, origin: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_bounds(&Host, world, data, origin)
}

#[wasm_bindgen]
pub fn og_terrain_edit(world: u32, data: f64, brush: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_edit(&Host, world, data, brush)
}

#[wasm_bindgen]
pub fn og_terrain_layer(world: u32, data: f64, index: u32) -> String {
    <Host as TerrainApi>::og_terrain_layer(&Host, world, data, index)
}

#[wasm_bindgen]
pub fn og_terrain_set_layer(world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    <Host as TerrainApi>::og_terrain_set_layer(&Host, world, data, index, layer)
}
