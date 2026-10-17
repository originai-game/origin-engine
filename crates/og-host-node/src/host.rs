//! The trait implementations of the native host: every binding symbol of
//! `og_api::host`. A member the IDL gains and this file lacks is a build
//! error. Most bodies are the host-neutral runtime; views, entities and the
//! dispatcher are where the native host differs.

use crate::{NodeArray, NodeFunction, NodeHooks, NodeHost, NodeViews};
use og_api::host::*;
use og_api::runtime as rt;
use std::rc::Rc;

impl Wire for NodeHost {
    type Array = NodeArray;
    type Views = NodeViews;
    type Function = NodeFunction;
}

impl Lifecycle for NodeHost {
    fn og_host_set_dispatcher(&self, dispatcher: Self::Function) {
        rt::set_hooks(Rc::new(NodeHooks { dispatcher }));
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
    fn og_host_begin_bundle(&self) {
        rt::og_host_begin_bundle()
    }
    fn og_engine_call(&self, engine: u32, frame: String) -> String {
        rt::og_engine_call(engine, frame)
    }
    fn og_engine_take_outputs(&self, engine: u32) -> String {
        rt::og_engine_take_outputs(engine)
    }
    fn og_engine_take_log(&self, engine: u32) -> String {
        rt::og_engine_take_log(engine)
    }
}

impl ReflectApi for NodeHost {
    fn og_reflect_define_component(&self, name: String, fields: String) -> String {
        rt::og_reflect_define_component(name, fields)
    }
    fn og_reflect_define_resource(&self, name: String, fields: String) -> String {
        rt::og_reflect_define_resource(name, fields)
    }
}

impl WorldApi for NodeHost {
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
        NodeArray::new(rt::view_entities(archetype_view))
    }
    fn og_archetype_view_entity(&self, archetype_view: u32, index: u32) -> f64 {
        rt::og_archetype_view_entity(archetype_view, index)
    }
    fn og_archetype_view_has(&self, archetype_view: u32, component: String) -> bool {
        rt::og_archetype_view_has(archetype_view, component)
    }
    fn og_archetype_view_column(&self, archetype_view: u32, component: String) -> Self::Views {
        // One ArrayBuffer per field over the column (or a copy of it, see
        // `NodeViews`). Empty is a refusal.
        NodeViews(rt::view_columns(archetype_view, &component).unwrap_or_default())
    }
}

impl SystemApi for NodeHost {
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

impl GameApi for NodeHost {
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

impl TransformApi for NodeHost {
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

impl RngApi for NodeHost {
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

impl InputApi for NodeHost {
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
}

impl CameraApi for NodeHost {
    fn og_camera_look_at(&self, world: u32, target: String) {
        rt::og_camera_look_at(world, target)
    }
    fn og_camera_world_to_screen(&self, world: u32, point: String) -> String {
        rt::og_camera_world_to_screen(world, point)
    }
    fn og_camera_screen_to_ray(&self, world: u32, x: f64, y: f64) -> String {
        rt::og_camera_screen_to_ray(world, x, y)
    }
}

impl RenderApi for NodeHost {
    fn og_render_standard_material(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_standard_material(world, desc)
    }
    fn og_render_update_material(&self, world: u32, material: f64, desc: Option<String>) -> String {
        rt::og_render_update_material(world, material, desc)
    }
    fn og_render_capabilities(&self, world: u32) -> String {
        rt::og_render_capabilities(world)
    }
    fn og_render_text(&self, world: u32, desc: Option<String>) -> String {
        rt::og_render_text(world, desc)
    }
}

impl LoadApi for NodeHost {
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
    fn og_load_sound(&self, world: u32, path: String) -> String {
        rt::og_load_sound(world, path)
    }
    fn og_load_sound_info(&self, world: u32, sound: f64) -> String {
        rt::og_load_sound_info(world, sound)
    }
    fn og_load_error(&self, world: u32, asset: f64) -> String {
        rt::og_load_error(world, asset)
    }
}

impl CommandsApi for NodeHost {
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

impl ProbeApi for NodeHost {
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
}
