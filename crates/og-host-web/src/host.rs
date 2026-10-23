//! The trait implementations of the web host: every binding symbol of
//! `og_api::host`. A member the IDL gains and this file lacks is a build
//! error. Most bodies are the host-neutral runtime; views, entities and the
//! dispatcher are where the web differs.

use crate::{WebHooks, WebHost};
use og_api::host::*;
use og_runtime as rt;
use std::rc::Rc;

impl Wire for WebHost {
    /// Copied into a fresh `Uint32Array` by wasm-bindgen.
    type Array = Vec<u32>;
    /// Byte offsets of each field's column in `memory.buffer`.
    type Views = Vec<u32>;
    type Function = js_sys::Function;
    /// Copied by wasm-bindgen: a fresh `Uint8Array` out, a copy of one in.
    type Bytes = Vec<u8>;
}

impl Lifecycle for WebHost {
    fn og_host_set_dispatcher(&self, dispatcher: Self::Function) {
        rt::set_hooks(Rc::new(WebHooks { dispatcher }));
        rt::set_physics_clock(crate::clock_ms);
    }
    fn og_engine_create(&self, game: u32, options: String) -> String {
        rt::og_engine_create(game, options)
    }
    fn og_engine_destroy(&self, engine: u32) -> bool {
        rt::og_engine_destroy(engine)
    }
    fn og_engine_advance(&self, engine: u32, steps: u32) -> String {
        rt::og_engine_advance(engine, steps)
    }
    fn og_engine_command(&self, engine: u32, action: String, args: String) -> String {
        rt::og_engine_command(engine, action, args)
    }
    fn og_engine_frame(&self, engine: u32, elapsed_ms: f64) -> String {
        rt::og_engine_frame(engine, elapsed_ms)
    }
    fn og_engine_write_resource(&self, world: u32, resource: String, value: String) -> String {
        rt::og_engine_write_resource(world, resource, value)
    }
    fn og_engine_resource_layout(&self, world: u32, resource: String) -> String {
        rt::og_engine_resource_layout(world, resource)
    }
    fn og_engine_read_resource(&self, world: u32, resource: String) -> Self::Bytes {
        rt::og_engine_read_resource(world, resource)
    }
    fn og_engine_write_resource_bytes(
        &self,
        world: u32,
        resource: String,
        bytes: Self::Bytes,
    ) -> String {
        rt::og_engine_write_resource_bytes(world, resource, &bytes)
    }
    fn og_host_begin_bundle(&self) {
        rt::og_host_begin_bundle()
    }
    fn og_engine_call(&self, engine: u32, frame: String) -> String {
        rt::og_engine_call(engine, frame)
    }
    fn og_engine_take_outputs(&self, engine: u32) -> String {
        rt::og_engine_take_outputs(engine)
    }
    fn og_platform_take(&self, engine: u32) -> String {
        rt::og_platform_take(engine)
    }
    fn og_platform_answer(&self, engine: u32, request: u32, answer: String) -> bool {
        rt::og_platform_answer(engine, request, answer)
    }
    fn og_platform_set_host(&self, engine: u32, info: String) {
        rt::og_platform_set_host(engine, info)
    }
    fn og_room_take(&self, engine: u32) -> String {
        rt::og_room_take(engine)
    }
    fn og_room_deliver(&self, engine: u32, event: String) -> bool {
        rt::og_room_deliver(engine, event)
    }
    fn og_room_set_host(&self, engine: u32, info: String) {
        rt::og_room_set_host(engine, info)
    }
    fn og_engine_take_log(&self, engine: u32) -> String {
        rt::og_engine_take_log(engine)
    }
}

impl ReflectApi for WebHost {
    fn og_reflect_define_component(&self, name: String, fields: String) -> String {
        rt::og_reflect_define_component(name, fields)
    }
    fn og_reflect_define_resource(&self, name: String, fields: String) -> String {
        rt::og_reflect_define_resource(name, fields)
    }
}

impl WorldApi for WebHost {
    fn og_world_get_tick(&self, world: u32) -> u32 {
        rt::og_world_get_tick(world)
    }
    fn og_world_spawn(&self, world: u32, desc: Option<String>) -> String {
        rt::og_world_spawn(world, desc)
    }
    fn og_world_despawn(&self, world: u32, entity: f64) -> String {
        rt::og_world_despawn(world, entity)
    }
    fn og_world_alive(&self, world: u32, entity: f64) -> bool {
        rt::og_world_alive(world, entity)
    }
    fn og_world_has(&self, world: u32, entity: f64, component: String) -> bool {
        rt::og_world_has(world, entity, component)
    }
    fn og_world_insert(
        &self,
        world: u32,
        entity: f64,
        component: String,
        value: Option<String>,
    ) -> String {
        rt::og_world_insert(world, entity, component, value)
    }
    fn og_world_remove(&self, world: u32, entity: f64, component: String) -> String {
        rt::og_world_remove(world, entity, component)
    }
    fn og_world_get(&self, world: u32, entity: f64, component: String) -> String {
        rt::og_world_get(world, entity, component)
    }
    fn og_world_set(&self, world: u32, entity: f64, component: String, value: String) -> String {
        rt::og_world_set(world, entity, component, value)
    }
    fn og_world_resource(&self, world: u32, resource: String) -> String {
        rt::og_world_resource(world, resource)
    }
    fn og_world_find(&self, world: u32, name: String) -> String {
        rt::og_world_find(world, name)
    }
    fn og_world_name_of(&self, world: u32, entity: f64) -> String {
        rt::og_world_name_of(world, entity)
    }
    fn og_world_query(&self, world: u32, desc: Option<String>) -> String {
        rt::og_world_query(world, desc)
    }
    fn og_world_events(&self, world: u32, event: String) -> String {
        rt::og_world_events(world, event)
    }
    fn og_world_emit(&self, world: u32, event: String, payload: String) -> String {
        rt::og_world_emit(world, event, payload)
    }
    fn og_world_instantiate(&self, world: u32, asset: f64, desc: Option<String>) -> String {
        rt::og_world_instantiate(world, asset, desc)
    }
    fn og_world_snapshot(&self, world: u32) -> String {
        rt::og_world_snapshot(world)
    }
    fn og_world_restore(&self, world: u32, save: String, desc: Option<String>) -> String {
        rt::og_world_restore(world, save, desc)
    }
    fn og_query_archetypes(&self, query: u32) -> String {
        rt::og_query_archetypes(query)
    }
    fn og_query_count(&self, query: u32) -> u32 {
        rt::og_query_count(query)
    }
    fn og_archetype_view_get_count(&self, archetype_view: u32) -> u32 {
        rt::og_archetype_view_get_count(archetype_view)
    }
    fn og_archetype_view_get_tick(&self, archetype_view: u32) -> u32 {
        rt::og_archetype_view_get_tick(archetype_view)
    }
    fn og_archetype_view_get_entities(&self, archetype_view: u32) -> Self::Array {
        rt::view_entities(archetype_view)
    }
    fn og_archetype_view_entity(&self, archetype_view: u32, index: u32) -> f64 {
        rt::og_archetype_view_entity(archetype_view, index)
    }
    fn og_archetype_view_has(&self, archetype_view: u32, component: String) -> bool {
        rt::og_archetype_view_has(archetype_view, component)
    }
    fn og_archetype_view_column(&self, archetype_view: u32, component: String) -> Self::Views {
        // Byte offsets into the wasm memory, one per field; the loader makes
        // typed arrays over `memory.buffer` at them. Empty is a refusal.
        rt::view_columns(archetype_view, &component)
            .map(|cols| cols.iter().map(|c| c.ptr as usize as u32).collect())
            .unwrap_or_default()
    }
}

impl SystemApi for WebHost {
    fn og_system_get_name(&self, system: u32) -> String {
        rt::og_system_get_name(system)
    }
    fn og_system_get_reads(&self, system: u32) -> String {
        rt::og_system_get_reads(system)
    }
    fn og_system_get_writes(&self, system: u32) -> String {
        rt::og_system_get_writes(system)
    }
    fn og_system_define_system(&self, name: String, desc: String, run: u32) -> u32 {
        rt::og_system_define_system(name, desc, run)
    }
}

impl GameApi for WebHost {
    fn og_game_get_tick(&self, game: u32) -> u32 {
        rt::og_game_get_tick(game)
    }
    fn og_game_get_systems(&self, game: u32) -> String {
        rt::og_game_get_systems(game)
    }
    fn og_game_game(&self, desc: Option<String>) -> u32 {
        rt::og_game_game(desc)
    }
}

impl TransformApi for WebHost {
    fn og_transform_compose(&self, position: String, quaternion: String, scale: String) -> String {
        rt::og_transform_compose(position, quaternion, scale)
    }
    fn og_transform_decompose(&self, matrix: String) -> String {
        rt::og_transform_decompose(matrix)
    }
    fn og_transform_look_at(&self, eye: String, target: String, up: Option<String>) -> String {
        rt::og_transform_look_at(eye, target, up)
    }
    fn og_transform_from_axis_angle(&self, axis: String, angle: f64) -> String {
        rt::og_transform_from_axis_angle(axis, angle)
    }
}

impl RngApi for WebHost {
    fn og_rng_stream_get_seed(&self, rng_stream: u32) -> u32 {
        rt::og_rng_stream_get_seed(rng_stream)
    }
    fn og_rng_stream_next(&self, rng_stream: u32) -> f64 {
        rt::og_rng_stream_next(rng_stream)
    }
    fn og_rng_stream_range(&self, rng_stream: u32, low: f64, high: f64) -> f64 {
        rt::og_rng_stream_range(rng_stream, low, high)
    }
    fn og_rng_stream_integer(&self, rng_stream: u32, low: i32, high: i32) -> i32 {
        rt::og_rng_stream_integer(rng_stream, low, high)
    }
    fn og_rng_stream_chance(&self, rng_stream: u32, p: f64) -> bool {
        rt::og_rng_stream_chance(rng_stream, p)
    }
    fn og_rng_stream_fork(&self, rng_stream: u32, label: String) -> u32 {
        rt::og_rng_stream_fork(rng_stream, label)
    }
    fn og_rng_next(&self, world: u32) -> f64 {
        rt::og_rng_next(world)
    }
    fn og_rng_range(&self, world: u32, low: f64, high: f64) -> f64 {
        rt::og_rng_range(world, low, high)
    }
    fn og_rng_integer(&self, world: u32, low: i32, high: i32) -> i32 {
        rt::og_rng_integer(world, low, high)
    }
    fn og_rng_chance(&self, world: u32, p: f64) -> bool {
        rt::og_rng_chance(world, p)
    }
    fn og_rng_fork(&self, world: u32, label: String) -> u32 {
        rt::og_rng_fork(world, label)
    }
}

impl InputApi for WebHost {
    fn og_input_down(&self, world: u32, key: String) -> bool {
        rt::og_input_down(world, key)
    }
    fn og_input_pressed(&self, world: u32, key: String) -> bool {
        rt::og_input_pressed(world, key)
    }
    fn og_input_released(&self, world: u32, key: String) -> bool {
        rt::og_input_released(world, key)
    }
    fn og_input_axis(&self, world: u32, negative: String, positive: String) -> f64 {
        rt::og_input_axis(world, negative, positive)
    }
    fn og_input_axis2(
        &self,
        world: u32,
        left: String,
        right: String,
        down: String,
        up: String,
    ) -> String {
        rt::og_input_axis2(world, left, right, down, up)
    }
    fn og_input_button_down(&self, world: u32, button: String) -> bool {
        rt::og_input_button_down(world, button)
    }
    fn og_input_button_pressed(&self, world: u32, button: String) -> bool {
        rt::og_input_button_pressed(world, button)
    }
    fn og_input_button_released(&self, world: u32, button: String) -> bool {
        rt::og_input_button_released(world, button)
    }
    fn og_pointers_down(&self, world: u32, id: u32) -> bool {
        rt::og_pointers_down(world, id)
    }
    fn og_pointers_pressed(&self, world: u32, id: u32) -> bool {
        rt::og_pointers_pressed(world, id)
    }
    fn og_pointers_released(&self, world: u32, id: u32) -> bool {
        rt::og_pointers_released(world, id)
    }
    fn og_pointers_position(&self, world: u32, id: u32) -> String {
        rt::og_pointers_position(world, id)
    }
    fn og_pointers_start(&self, world: u32, id: u32) -> String {
        rt::og_pointers_start(world, id)
    }
    fn og_pointers_count(&self, world: u32) -> u32 {
        rt::og_pointers_count(world)
    }
    fn og_gamepad_connected(&self, world: u32, pad: Option<u32>) -> bool {
        rt::og_gamepad_connected(world, pad)
    }
    fn og_gamepad_down(&self, world: u32, button: String, pad: Option<u32>) -> bool {
        rt::og_gamepad_down(world, button, pad)
    }
    fn og_gamepad_pressed(&self, world: u32, button: String, pad: Option<u32>) -> bool {
        rt::og_gamepad_pressed(world, button, pad)
    }
    fn og_gamepad_released(&self, world: u32, button: String, pad: Option<u32>) -> bool {
        rt::og_gamepad_released(world, button, pad)
    }
    fn og_gamepad_axis(&self, world: u32, axis: String, pad: Option<u32>) -> f64 {
        rt::og_gamepad_axis(world, axis, pad)
    }
}

impl CameraApi for WebHost {
    fn og_camera_look_at(&self, world: u32, target: String) {
        rt::og_camera_look_at(world, target)
    }
    fn og_camera_world_to_screen(&self, world: u32, point: String) -> String {
        rt::og_camera_world_to_screen(world, point)
    }
    fn og_camera_screen_to_ray(&self, world: u32, x: f64, y: f64) -> String {
        rt::og_camera_screen_to_ray(world, x, y)
    }
    fn og_camera_shake(&self, world: u32, trauma: f64) -> String {
        rt::og_camera_shake(world, trauma)
    }
    fn og_camera_punch(&self, world: u32, amount: f64) -> String {
        rt::og_camera_punch(world, amount)
    }
    fn og_camera_kick(&self, world: u32, direction: String, distance: f64) -> String {
        rt::og_camera_kick(world, direction, distance)
    }
}

impl RenderApi for WebHost {
    fn og_render_shader_material(&self, world: u32, shader: f64, desc: Option<String>) -> String {
        rt::og_render_shader_material(world, shader, desc)
    }
    fn og_render_set_material_shader(&self, world: u32, material: f64, shader: String) -> String {
        rt::og_render_set_material_shader(world, material, shader)
    }
    fn og_render_set_shader_param(
        &self,
        world: u32,
        material: f64,
        name: String,
        value: String,
    ) -> String {
        rt::og_render_set_shader_param(world, material, name, value)
    }
    fn og_render_box_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_box_geometry(world, desc)
    }
    fn og_render_plane_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_plane_geometry(world, desc)
    }
    fn og_render_sphere_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_sphere_geometry(world, desc)
    }
    fn og_render_buffer_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_buffer_geometry(world, desc)
    }
    fn og_render_update_geometry(&self, world: u32, geometry: f64, desc: Option<String>) -> String {
        rt::og_render_update_geometry(world, geometry, desc)
    }
    fn og_render_geometry_info(&self, world: u32, geometry: f64) -> String {
        rt::og_render_geometry_info(world, geometry)
    }
    fn og_render_cylinder_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_cylinder_geometry(world, desc)
    }
    fn og_render_cone_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_cone_geometry(world, desc)
    }
    fn og_render_capsule_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_capsule_geometry(world, desc)
    }
    fn og_render_torus_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_torus_geometry(world, desc)
    }
    fn og_render_ring_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_ring_geometry(world, desc)
    }
    fn og_render_circle_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_circle_geometry(world, desc)
    }
    fn og_render_tube_geometry(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_tube_geometry(world, desc)
    }
    fn og_render_lines(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_lines(world, desc)
    }
    fn og_render_points(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_points(world, desc)
    }
    fn og_render_raycast(&self, world: u32, ray: String, desc: Option<String>) -> String {
        rt::og_render_raycast(world, ray, desc)
    }
    fn og_render_baked_lighting(&self, world: u32) -> String {
        rt::og_render_baked_lighting(world)
    }
    fn og_render_sprite(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_sprite(world, desc)
    }
    fn og_render_measure_text(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_measure_text(world, desc)
    }
    fn og_render_standard_material(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_standard_material(world, desc)
    }
    fn og_render_update_material(&self, world: u32, material: f64, desc: Option<String>) -> String {
        rt::og_render_update_material(world, material, desc)
    }
    fn og_render_capabilities(&self, world: u32) -> String {
        rt::og_render_capabilities(world)
    }
    fn og_render_apply_look(&self, world: u32, look: f64) -> String {
        rt::og_render_apply_look(world, look)
    }
    fn og_render_look(&self, world: u32) -> String {
        rt::og_render_look(world)
    }
    fn og_render_text(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_text(world, desc)
    }
    fn og_render_ribbon(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_ribbon(world, desc)
    }
    fn og_render_floating_text(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_floating_text(world, desc)
    }
}

impl LoadApi for WebHost {
    fn og_load_shader(&self, world: u32, path: String) -> String {
        rt::og_load_shader(world, path)
    }
    fn og_load_texture(&self, world: u32, path: String) -> String {
        rt::og_load_texture(world, path)
    }
    fn og_load_gltf(&self, world: u32, path: String, desc: Option<String>) -> String {
        rt::og_load_gltf(world, path, desc)
    }
    fn og_load_scene(&self, world: u32, path: String) -> String {
        rt::og_load_scene(world, path)
    }
    fn og_load_status(&self, world: u32, asset: f64) -> String {
        rt::og_load_status(world, asset)
    }
    fn og_load_gltf_info(&self, world: u32, asset: f64) -> String {
        rt::og_load_gltf_info(world, asset)
    }
    fn og_load_bundle(&self, world: u32, path: String) -> String {
        rt::og_load_bundle(world, path)
    }
    fn og_load_bundle_info(&self, world: u32, asset: f64) -> String {
        rt::og_load_bundle_info(world, asset)
    }
    fn og_load_default_bundle(&self, world: u32) -> String {
        rt::og_load_default_bundle(world)
    }
    fn og_load_font(&self, world: u32, path: String) -> String {
        rt::og_load_font(world, path)
    }
    fn og_load_font_info(&self, world: u32, font: f64) -> String {
        rt::og_load_font_info(world, font)
    }
    fn og_load_animation(&self, world: u32, path: String) -> String {
        rt::og_load_animation(world, path)
    }
    fn og_load_timeline(&self, world: u32, path: String) -> String {
        rt::og_load_timeline(world, path)
    }
    fn og_load_sound(&self, world: u32, path: String) -> String {
        rt::og_load_sound(world, path)
    }
    fn og_load_sound_info(&self, world: u32, sound: f64) -> String {
        rt::og_load_sound_info(world, sound)
    }
    fn og_load_video(&self, world: u32, path: String) -> String {
        rt::og_load_video(world, path)
    }
    fn og_load_video_info(&self, world: u32, video: f64) -> String {
        rt::og_load_video_info(world, video)
    }
    fn og_load_error(&self, world: u32, asset: f64) -> String {
        rt::og_load_error(world, asset)
    }
    fn og_load_lut(&self, world: u32, path: String) -> String {
        rt::og_load_lut(world, path)
    }
    fn og_load_environment(&self, world: u32, path: String) -> String {
        rt::og_load_environment(world, path)
    }

    fn og_load_texture_array(&self, world: u32, path: String) -> String {
        rt::og_load_texture_array(world, path)
    }
    fn og_load_emitter(&self, world: u32, path: String) -> String {
        rt::og_load_emitter(world, path)
    }
    fn og_load_skin(&self, world: u32, path: String) -> String {
        rt::og_load_skin(world, path)
    }
    fn og_load_look(&self, world: u32, path: String) -> String {
        rt::og_load_look(world, path)
    }
    fn og_load_look_info(&self, world: u32, look: f64) -> String {
        rt::og_load_look_info(world, look)
    }
    fn og_load_palette(&self, world: u32, path: String) -> String {
        rt::og_load_palette(world, path)
    }
    fn og_load_flipbook(&self, world: u32, path: String) -> String {
        rt::og_load_flipbook(world, path)
    }

    fn og_load_rig2d(&self, world: u32, path: String) -> String {
        rt::og_load_rig2d(world, path)
    }

    fn og_load_rig(&self, world: u32, path: String) -> String {
        rt::og_load_rig(world, path)
    }
    fn og_load_tilemap(&self, world: u32, path: String) -> String {
        rt::og_load_tilemap(world, path)
    }
}

impl TimeApi for WebHost {
    fn og_time_set_scale(&self, world: u32, scale: f64) -> String {
        rt::og_time_set_scale(world, scale)
    }
    fn og_time_hit_stop(&self, world: u32, ticks: u32) -> String {
        rt::og_time_hit_stop(world, ticks)
    }
}

impl TweenApi for WebHost {
    fn og_tween_to(&self, world: u32, desc: Option<String>) -> String {
        rt::og_tween_to(world, desc)
    }
    fn og_tween_sequence(&self, world: u32, steps: String, desc: Option<String>) -> String {
        rt::og_tween_sequence(world, steps, desc)
    }
    fn og_tween_parallel(&self, world: u32, tweens: String, desc: Option<String>) -> String {
        rt::og_tween_parallel(world, tweens, desc)
    }
    fn og_tween_cancel(&self, world: u32, tween: f64) -> bool {
        rt::og_tween_cancel(world, tween)
    }
    fn og_tween_cancel_entity(&self, world: u32, entity: f64) -> u32 {
        rt::og_tween_cancel_entity(world, entity)
    }
    fn og_tween_active(&self, world: u32, tween: f64) -> bool {
        rt::og_tween_active(world, tween)
    }
    fn og_tween_squash(&self, world: u32, desc: Option<String>) -> String {
        rt::og_tween_squash(world, desc)
    }
    fn og_tween_flash(&self, world: u32, desc: Option<String>) -> String {
        rt::og_tween_flash(world, desc)
    }
}

impl TilemapApi for WebHost {
    fn og_tilemap_set_tile(&self, world: u32, desc: Option<String>) -> String {
        rt::og_tilemap_set_tile(world, desc)
    }
    fn og_tilemap_get_tile(&self, world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32 {
        rt::og_tilemap_get_tile(world, entity, layer, x, y)
    }
    fn og_tilemap_cell_at(&self, world: u32, entity: f64, point: String) -> String {
        rt::og_tilemap_cell_at(world, entity, point)
    }
    fn og_tilemap_cell_center(&self, world: u32, entity: f64, x: u32, y: u32) -> String {
        rt::og_tilemap_cell_center(world, entity, x, y)
    }
}

impl AnimApi for WebHost {
    fn og_anim_graph(&self, world: u32, desc: Option<String>) -> String {
        rt::og_anim_graph(world, desc)
    }

    fn og_anim_humanoid(&self, world: u32, desc: Option<String>) -> String {
        rt::og_anim_humanoid(world, desc)
    }

    fn og_anim_rig2d_info(&self, world: u32, rig: f64) -> String {
        rt::og_anim_rig2d_info(world, rig)
    }

    fn og_anim_retarget(&self, world: u32, desc: Option<String>) -> String {
        rt::og_anim_retarget(world, desc)
    }
}

impl AtmosphereApi for WebHost {
    fn og_atmosphere_time_of_day(&self, world: u32) -> f64 {
        rt::og_atmosphere_time_of_day(world)
    }
}

impl PhysicsApi for WebHost {
    fn og_physics_rope_points(&self, world: u32, entity: f64) -> String {
        rt::og_physics_rope_points(world, entity)
    }
    fn og_physics_cloth_points(&self, world: u32, entity: f64) -> String {
        rt::og_physics_cloth_points(world, entity)
    }
    fn og_physics_fluid_particles(&self, world: u32, entity: f64) -> String {
        rt::og_physics_fluid_particles(world, entity)
    }
    fn og_physics_fluid3d_particles(&self, world: u32, entity: f64) -> String {
        rt::og_physics_fluid3d_particles(world, entity)
    }
    fn og_physics_water_height(&self, world: u32, entity: f64, x: f64, z: f64) -> String {
        rt::og_physics_water_height(world, entity, x, z)
    }
    fn og_physics_vehicle_wheels(&self, world: u32, entity: f64) -> String {
        rt::og_physics_vehicle_wheels(world, entity)
    }
    fn og_physics_apply_force(
        &self,
        world: u32,
        entity: f64,
        force: String,
        point: Option<String>,
    ) -> String {
        rt::og_physics_apply_force(world, entity, force, point)
    }
    fn og_physics_apply_impulse(
        &self,
        world: u32,
        entity: f64,
        impulse: String,
        point: Option<String>,
    ) -> String {
        rt::og_physics_apply_impulse(world, entity, impulse, point)
    }
    fn og_physics_apply_torque(&self, world: u32, entity: f64, torque: String) -> String {
        rt::og_physics_apply_torque(world, entity, torque)
    }
    fn og_physics_apply_torque_impulse(&self, world: u32, entity: f64, torque: String) -> String {
        rt::og_physics_apply_torque_impulse(world, entity, torque)
    }
    fn og_physics_ragdoll_rig_from_skeleton(&self, world: u32, skeleton: f64) -> String {
        rt::og_physics_ragdoll_rig_from_skeleton(world, skeleton)
    }
    fn og_physics_set_ragdoll_rig(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_physics_set_ragdoll_rig(world, entity, desc)
    }
    fn og_physics_ragdoll_pose(&self, world: u32, entity: f64) -> String {
        rt::og_physics_ragdoll_pose(world, entity)
    }
    fn og_physics_stats(&self, world: u32) -> String {
        rt::og_physics_stats(world)
    }
    fn og_physics_raycast(&self, world: u32, desc: Option<String>) -> String {
        rt::og_physics_raycast(world, desc)
    }
    fn og_physics_shape_cast(&self, world: u32, desc: Option<String>) -> String {
        rt::og_physics_shape_cast(world, desc)
    }
    fn og_physics_overlap(&self, world: u32, desc: Option<String>) -> String {
        rt::og_physics_overlap(world, desc)
    }
    fn og_physics_set_mesh(&self, world: u32, mesh: f64, desc: Option<String>) -> String {
        rt::og_physics_set_mesh(world, mesh, desc)
    }
}

impl UiApi for WebHost {
    fn og_ui_configure(&self, world: u32, desc: Option<String>) -> String {
        rt::og_ui_configure(world, desc)
    }
    fn og_ui_viewport(&self, world: u32) -> String {
        rt::og_ui_viewport(world)
    }
    fn og_ui_theme(&self, world: u32) -> String {
        rt::og_ui_theme(world)
    }
    fn og_ui_open(&self, id: String, desc: Option<String>) {
        rt::og_ui_open(id, desc)
    }
    fn og_ui_open_panel(&self, id: String, desc: Option<String>) {
        rt::og_ui_open_panel(id, desc)
    }
    fn og_ui_open_scroll(&self, id: String, desc: Option<String>) {
        rt::og_ui_open_scroll(id, desc)
    }
    fn og_ui_close(&self) -> String {
        rt::og_ui_close()
    }
    fn og_ui_label(&self, id: String, text: String, desc: Option<String>) {
        rt::og_ui_label(id, text, desc)
    }
    fn og_ui_button(&self, id: String, text: String, desc: Option<String>) -> String {
        rt::og_ui_button(id, text, desc)
    }
    fn og_ui_image(&self, id: String, desc: Option<String>) {
        rt::og_ui_image(id, desc)
    }
    fn og_ui_progress(&self, id: String, value: f64, desc: Option<String>) {
        rt::og_ui_progress(id, value, desc)
    }
    fn og_ui_slider(&self, id: String, value: f64, desc: Option<String>) -> String {
        rt::og_ui_slider(id, value, desc)
    }
    fn og_ui_toggle(&self, id: String, on: bool, label: String) -> String {
        rt::og_ui_toggle(id, on, label)
    }
    fn og_ui_text_field(&self, id: String, value: String, desc: Option<String>) -> String {
        rt::og_ui_text_field(id, value, desc)
    }
    fn og_ui_text_area(&self, id: String, value: String, desc: Option<String>) -> String {
        rt::og_ui_text_area(id, value, desc)
    }
    fn og_ui_tooltip(&self, text: String) {
        rt::og_ui_tooltip(text)
    }
    fn og_ui_breakpoint(&self) -> String {
        rt::og_ui_breakpoint()
    }
    fn og_ui_frame_theme(&self) -> String {
        rt::og_ui_frame_theme()
    }
    fn og_ui_frame_viewport(&self) -> String {
        rt::og_ui_frame_viewport()
    }
    fn og_ui_has_icon(&self, name: String) -> bool {
        rt::og_ui_has_icon(name)
    }
    fn og_ui_seconds(&self) -> f64 {
        rt::og_ui_seconds()
    }
    fn og_ui_counter(&self, id: String, value: f64, desc: Option<String>) -> f64 {
        rt::og_ui_counter(id, value, desc)
    }
    fn og_ui_burst(&self, desc: Option<String>) {
        rt::og_ui_burst(desc)
    }
    fn og_ui_shake(&self, desc: Option<String>) {
        rt::og_ui_shake(desc)
    }
    fn og_ui_screen_transition(&self, id: String, screen: String, desc: Option<String>) -> String {
        rt::og_ui_screen_transition(id, screen, desc)
    }
    fn og_ui_open_anchor(&self, id: String, desc: Option<String>) {
        rt::og_ui_open_anchor(id, desc)
    }
    fn og_ui_open_transition(&self, id: String, show: bool, desc: Option<String>) -> String {
        rt::og_ui_open_transition(id, show, desc)
    }
    fn og_ui_open_modal(&self, id: String, open: bool, desc: Option<String>) -> String {
        rt::og_ui_open_modal(id, open, desc)
    }
    fn og_ui_close_modal(&self) {
        rt::og_ui_close_modal()
    }
    fn og_ui_spotlight(&self, id: String, hole: String, desc: Option<String>) -> String {
        rt::og_ui_spotlight(id, hole, desc)
    }
    fn og_ui_bounds(&self, path: String) -> String {
        rt::og_ui_bounds(path)
    }
    fn og_ui_open_list(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_open_list(id, desc)
    }
    fn og_ui_open_grid(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_open_grid(id, desc)
    }
    fn og_ui_list_item(&self, index: u32, selected: bool) -> String {
        rt::og_ui_list_item(index, selected)
    }
    fn og_ui_grid_item(&self, index: u32, selected: bool) -> String {
        rt::og_ui_grid_item(index, selected)
    }
    fn og_ui_close_list(&self) -> String {
        rt::og_ui_close_list()
    }
    fn og_ui_dragging(&self, id: String) -> String {
        rt::og_ui_dragging(id)
    }
    fn og_ui_tabs(&self, id: String, items: String, selected: u32, desc: Option<String>) -> String {
        rt::og_ui_tabs(id, items, selected, desc)
    }
    fn og_ui_select(
        &self,
        id: String,
        options: String,
        selected: u32,
        desc: Option<String>,
    ) -> String {
        rt::og_ui_select(id, options, selected, desc)
    }
    fn og_ui_stepper(&self, id: String, value: f64, desc: Option<String>) -> String {
        rt::og_ui_stepper(id, value, desc)
    }
    fn og_ui_radio(
        &self,
        id: String,
        options: String,
        selected: u32,
        desc: Option<String>,
    ) -> String {
        rt::og_ui_radio(id, options, selected, desc)
    }
    fn og_ui_checkbox(&self, id: String, on: bool, label: String) -> String {
        rt::og_ui_checkbox(id, on, label)
    }
    fn og_ui_checkbox_list(&self, id: String, labels: String, values: String) -> String {
        rt::og_ui_checkbox_list(id, labels, values)
    }
    fn og_ui_chip(&self, id: String, text: String, desc: Option<String>) -> String {
        rt::og_ui_chip(id, text, desc)
    }
    fn og_ui_badge(&self, id: String, text: String, tone: String) {
        rt::og_ui_badge(id, text, tone)
    }
    fn og_ui_divider(&self, id: String, vertical: bool) {
        rt::og_ui_divider(id, vertical)
    }
    fn og_ui_spacer(&self, id: String, size: String) {
        rt::og_ui_spacer(id, size)
    }
    fn og_ui_key_value(&self, id: String, key: String, value: String, mono: bool) {
        rt::og_ui_key_value(id, key, value, mono)
    }
    fn og_ui_skeleton(&self, id: String, desc: Option<String>) {
        rt::og_ui_skeleton(id, desc)
    }
    fn og_ui_spinner(&self, id: String, size: f64) {
        rt::og_ui_spinner(id, size)
    }
    fn og_ui_icon(&self, id: String, name: String, desc: Option<String>) {
        rt::og_ui_icon(id, name, desc)
    }
    fn og_ui_meter(&self, id: String, value: f64, max: f64, desc: Option<String>) {
        rt::og_ui_meter(id, value, max, desc)
    }
    fn og_ui_cooldown(&self, id: String, remaining: f64, desc: Option<String>) -> String {
        rt::og_ui_cooldown(id, remaining, desc)
    }
    fn og_ui_minimap(&self, id: String, desc: Option<String>) {
        rt::og_ui_minimap(id, desc)
    }
    fn og_ui_zoom_view(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_zoom_view(id, desc)
    }
    fn og_ui_dialogue(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_dialogue(id, desc)
    }
    fn og_ui_joystick(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_joystick(id, desc)
    }
    fn og_ui_dpad(&self, id: String, desc: Option<String>) -> String {
        rt::og_ui_dpad(id, desc)
    }
    fn og_ui_action_button(&self, id: String, text: String, desc: Option<String>) -> String {
        rt::og_ui_action_button(id, text, desc)
    }
    fn og_ui_toast(&self, world: u32, text: String, desc: Option<String>) -> u32 {
        rt::og_ui_toast(world, text, desc)
    }
    fn og_ui_dismiss_toast(&self, world: u32, toast_id: u32) {
        rt::og_ui_dismiss_toast(world, toast_id)
    }
    fn og_ui_navigate(&self, world: u32, intent: String) {
        rt::og_ui_navigate(world, intent)
    }
    fn og_ui_back(&self) -> bool {
        rt::og_ui_back()
    }
    fn og_ui_set_navigation(&self, on: bool) {
        rt::og_ui_set_navigation(on)
    }
    fn og_ui_rich_text(&self, id: String, markup: String, desc: Option<String>) -> String {
        rt::og_ui_rich_text(id, markup, desc)
    }
    fn og_ui_set_players(&self, count: u32) {
        rt::og_ui_set_players(count)
    }
    fn og_ui_focus(&self, pad: u32) -> String {
        rt::og_ui_focus(pad)
    }

    fn og_ui_a11y(&self) -> String {
        rt::og_ui_a11y()
    }
    fn og_ui_memory(&self, id: String, initial: f64) -> f64 {
        rt::og_ui_memory(id, initial)
    }
    fn og_ui_remember(&self, id: String, value: f64) {
        rt::og_ui_remember(id, value)
    }
    fn og_ui_tick(&self) -> u32 {
        rt::og_ui_tick()
    }
    fn og_ui_preview_viewport(&self, world: u32, viewport: String) {
        rt::og_ui_preview_viewport(world, viewport)
    }
}

impl CommandsApi for WebHost {
    fn og_commands_register(&self, name: String, handler: u32, desc: Option<String>) -> String {
        rt::og_commands_register(name, handler, desc)
    }
    fn og_commands_parse(&self, script: String) -> String {
        rt::og_commands_parse(script)
    }
    fn og_commands_schedule(&self, world: u32, script: String) -> String {
        rt::og_commands_schedule(world, script)
    }
    fn og_commands_help(&self, world: u32) -> String {
        rt::og_commands_help(world)
    }
}

impl ProbeApi for WebHost {
    fn og_runtime_probe_get_version(&self, runtime_probe: u32) -> u32 {
        rt::og_runtime_probe_get_version(runtime_probe)
    }
    fn og_runtime_probe_tick(&self, runtime_probe: u32) -> u32 {
        rt::og_runtime_probe_tick(runtime_probe)
    }
    fn og_runtime_probe_state(&self, runtime_probe: u32) -> String {
        rt::og_runtime_probe_state(runtime_probe)
    }
    fn og_runtime_probe_advance(&self, runtime_probe: u32, steps: u32) -> u32 {
        rt::og_runtime_probe_advance(runtime_probe, steps)
    }
    fn og_runtime_probe_pause(&self, runtime_probe: u32) -> bool {
        rt::og_runtime_probe_pause(runtime_probe)
    }
    fn og_runtime_probe_resume(&self, runtime_probe: u32) -> bool {
        rt::og_runtime_probe_resume(runtime_probe)
    }
    fn og_runtime_probe_input(&self, runtime_probe: u32, event: String) -> bool {
        rt::og_runtime_probe_input(runtime_probe, event)
    }
    fn og_runtime_probe_accepted(&self, runtime_probe: u32) -> u32 {
        rt::og_runtime_probe_accepted(runtime_probe)
    }
    fn og_runtime_probe_pick(&self, runtime_probe: u32, x: f64, y: f64) -> String {
        rt::og_runtime_probe_pick(runtime_probe, x, y)
    }
    fn og_runtime_probe_entity(&self, runtime_probe: u32, id: String) -> String {
        rt::og_runtime_probe_entity(runtime_probe, id)
    }
    fn og_runtime_probe_bounds(&self, runtime_probe: u32, id: String) -> String {
        rt::og_runtime_probe_bounds(runtime_probe, id)
    }
    fn og_runtime_probe_patch(
        &self,
        runtime_probe: u32,
        id: String,
        parameter: String,
        value: String,
    ) -> bool {
        rt::og_runtime_probe_patch(runtime_probe, id, parameter, value)
    }
    fn og_runtime_probe_run(&self, runtime_probe: u32, program: String) -> String {
        rt::og_runtime_probe_run(runtime_probe, program)
    }
    fn og_engine_probe_hello(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_hello(engine_probe)
    }
    fn og_engine_probe_capture(
        &self,
        engine_probe: u32,
        width: Option<u32>,
        height: Option<u32>,
    ) -> String {
        rt::og_engine_probe_capture(engine_probe, width, height)
    }
    fn og_engine_probe_sweep(&self, engine_probe: u32, desc: Option<String>) -> String {
        rt::og_engine_probe_sweep(engine_probe, desc)
    }
    fn og_engine_probe_reload(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_reload(engine_probe)
    }
    fn og_engine_probe_snapshot(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_snapshot(engine_probe)
    }
    fn og_engine_probe_restore(&self, engine_probe: u32, id: String) -> bool {
        rt::og_engine_probe_restore(engine_probe, id)
    }
    fn og_engine_probe_dump(&self, engine_probe: u32, desc: Option<String>) -> String {
        rt::og_engine_probe_dump(engine_probe, desc)
    }
    fn og_engine_probe_commands(&self, engine_probe: u32, script: String) -> String {
        rt::og_engine_probe_commands(engine_probe, script)
    }

    fn og_engine_probe_listen(&self, engine_probe: u32, seconds: Option<f64>) -> String {
        rt::og_engine_probe_listen(engine_probe, seconds)
    }

    fn og_engine_probe_memory(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_memory(engine_probe)
    }

    fn og_engine_probe_describe(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_describe(engine_probe)
    }

    fn og_engine_probe_expect(&self, engine_probe: u32, what: String) -> String {
        rt::og_engine_probe_expect(engine_probe, what)
    }

    fn og_engine_probe_expectations(&self, engine_probe: u32) -> String {
        rt::og_engine_probe_expectations(engine_probe)
    }

    fn og_engine_probe_save_scene(&self, engine_probe: u32, write: Option<bool>) -> String {
        rt::og_engine_probe_save_scene(engine_probe, write)
    }
}
