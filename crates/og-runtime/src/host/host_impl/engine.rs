//! The engine's own bindings (Lifecycle, CommandsApi, ProbeApi, DebugApi, AssertApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_engine {
    ($host:ty) => {
        impl $crate::host_impl::api::Lifecycle for $host {
            fn og_host_set_dispatcher(&self, dispatcher: Self::Function) {
                <Self as $crate::host_impl::HostWire>::install(dispatcher);
            }
            fn og_engine_create(&self, game: u32, options: String) -> String {
                $crate::og_engine_create(game, options)
            }
            fn og_engine_destroy(&self, engine: u32) -> bool {
                $crate::og_engine_destroy(engine)
            }
            fn og_engine_advance(&self, engine: u32, steps: u32) -> String {
                $crate::og_engine_advance(engine, steps)
            }
            fn og_engine_command(&self, engine: u32, action: String, args: String) -> String {
                $crate::og_engine_command(engine, action, args)
            }
            fn og_engine_frame(&self, engine: u32, elapsed_ms: f64) -> String {
                $crate::og_engine_frame(engine, elapsed_ms)
            }
            fn og_engine_write_resource(
                &self,
                world: u32,
                resource: String,
                value: String,
            ) -> String {
                $crate::og_engine_write_resource(world, resource, value)
            }
            fn og_engine_resource_layout(&self, world: u32, resource: String) -> String {
                $crate::host_impl::answer($crate::og_engine_resource_layout(world, resource))
            }
            fn og_engine_read_resource(&self, world: u32, resource: String) -> Self::Bytes {
                <Self as $crate::host_impl::HostWire>::bytes_out($crate::og_engine_read_resource(
                    world, resource,
                ))
            }
            fn og_engine_write_resource_bytes(
                &self,
                world: u32,
                resource: String,
                bytes: Self::Bytes,
            ) -> String {
                $crate::og_engine_write_resource_bytes(world, resource, &bytes)
            }
            fn og_host_shared(&self) -> Self::Shared {
                <Self as $crate::host_impl::HostWire>::shared()
            }
            fn og_engine_component_id(&self, world: u32, component: String) -> u32 {
                $crate::og_engine_component_id(world, component)
            }
            fn og_engine_component_layout(&self, world: u32, component: String) -> String {
                $crate::host_impl::answer($crate::og_engine_component_layout(world, component))
            }
            fn og_engine_read_component(&self, world: u32, entity: f64, component: u32) -> u32 {
                $crate::og_engine_read_component(world, entity, component)
            }
            fn og_engine_write_component(
                &self,
                world: u32,
                entity: f64,
                component: u32,
                epoch: u32,
            ) -> u32 {
                $crate::og_engine_write_component(world, entity, component, epoch)
            }
            fn og_engine_query_archetypes(&self, query: u32, from: u32) -> u32 {
                $crate::og_engine_query_archetypes(query, from)
            }
            fn og_engine_view_column(&self, view: u32, component: u32) -> Self::Views {
                <Self as $crate::host_impl::HostWire>::views($crate::view_columns(view, component))
            }
            fn og_engine_ui_patch(&self, world: u32, bytes: Self::Bytes) -> Self::Bytes {
                <Self as $crate::host_impl::HostWire>::bytes_out($crate::og_engine_ui_patch(
                    world, &bytes,
                ))
            }
            fn og_host_begin_bundle(&self) {
                $crate::og_host_begin_bundle()
            }
            fn og_engine_call(&self, engine: u32, frame: String) -> String {
                $crate::og_engine_call(engine, frame)
            }
            fn og_engine_take_outputs(&self, engine: u32) -> String {
                $crate::og_engine_take_outputs(engine)
            }
            fn og_platform_take(&self, engine: u32) -> String {
                $crate::og_platform_take(engine)
            }
            fn og_platform_answer(&self, engine: u32, request: u32, answer: String) -> bool {
                $crate::og_platform_answer(engine, request, answer)
            }
            fn og_platform_notify(&self, engine: u32, notice: String) -> bool {
                $crate::og_platform_notify(engine, notice)
            }
            fn og_platform_set_host(&self, engine: u32, info: String) {
                $crate::og_platform_set_host(engine, info)
            }
            fn og_room_take(&self, engine: u32) -> String {
                $crate::og_room_take(engine)
            }
            fn og_room_deliver(&self, engine: u32, event: String) -> bool {
                $crate::og_room_deliver(engine, event)
            }
            fn og_room_set_host(&self, engine: u32, info: String) {
                $crate::og_room_set_host(engine, info)
            }
            fn og_engine_take_log(&self, engine: u32) -> String {
                $crate::og_engine_take_log(engine)
            }
        }

        impl $crate::host_impl::api::CommandsApi for $host {
            fn og_commands_register(
                &self,
                name: String,
                handler: u32,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_commands_register(name, handler, desc))
            }
            fn og_commands_parse(&self, script: String) -> String {
                $crate::host_impl::answer($crate::og_commands_parse(script))
            }
            fn og_commands_schedule(&self, world: u32, script: String) -> String {
                $crate::host_impl::answer($crate::og_commands_schedule(world, script))
            }
            fn og_commands_help(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_commands_help(world))
            }
        }

        impl $crate::host_impl::api::ProbeApi for $host {
            fn og_runtime_probe_get_version(&self, runtime_probe: u32) -> u32 {
                $crate::og_runtime_probe_get_version(runtime_probe)
            }
            fn og_runtime_probe_tick(&self, runtime_probe: u32) -> u32 {
                $crate::og_runtime_probe_tick(runtime_probe)
            }
            fn og_runtime_probe_state(&self, runtime_probe: u32) -> String {
                $crate::og_runtime_probe_state(runtime_probe)
            }
            fn og_runtime_probe_advance(&self, runtime_probe: u32, steps: u32) -> u32 {
                $crate::og_runtime_probe_advance(runtime_probe, steps)
            }
            fn og_runtime_probe_pause(&self, runtime_probe: u32) -> bool {
                $crate::og_runtime_probe_pause(runtime_probe)
            }
            fn og_runtime_probe_resume(&self, runtime_probe: u32) -> bool {
                $crate::og_runtime_probe_resume(runtime_probe)
            }
            fn og_runtime_probe_input(&self, runtime_probe: u32, event: String) -> bool {
                $crate::og_runtime_probe_input(runtime_probe, event)
            }
            fn og_runtime_probe_accepted(&self, runtime_probe: u32) -> u32 {
                $crate::og_runtime_probe_accepted(runtime_probe)
            }
            fn og_runtime_probe_pick(&self, runtime_probe: u32, x: f64, y: f64) -> String {
                $crate::og_runtime_probe_pick(runtime_probe, x, y)
            }
            fn og_runtime_probe_entity(&self, runtime_probe: u32, id: String) -> String {
                $crate::og_runtime_probe_entity(runtime_probe, id)
            }
            fn og_runtime_probe_bounds(&self, runtime_probe: u32, id: String) -> String {
                $crate::og_runtime_probe_bounds(runtime_probe, id)
            }
            fn og_runtime_probe_patch(
                &self,
                runtime_probe: u32,
                id: String,
                parameter: String,
                value: String,
            ) -> bool {
                $crate::og_runtime_probe_patch(runtime_probe, id, parameter, value)
            }
            fn og_runtime_probe_run(&self, runtime_probe: u32, program: String) -> String {
                $crate::og_runtime_probe_run(runtime_probe, program)
            }
            fn og_engine_probe_hello(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_hello(engine_probe)
            }
            fn og_engine_probe_capture(
                &self,
                engine_probe: u32,
                width: Option<u32>,
                height: Option<u32>,
                fit: Option<u32>,
                region: Option<String>,
            ) -> String {
                $crate::og_engine_probe_capture(engine_probe, width, height, fit, region)
            }
            fn og_engine_probe_marks(&self, engine_probe: u32, desc: Option<String>) -> String {
                $crate::og_engine_probe_marks(engine_probe, desc)
            }
            fn og_engine_probe_timeline(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_timeline(engine_probe)
            }
            fn og_engine_probe_sweep(&self, engine_probe: u32, desc: Option<String>) -> String {
                $crate::og_engine_probe_sweep(engine_probe, desc)
            }
            fn og_engine_probe_reload(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_reload(engine_probe)
            }
            fn og_engine_probe_snapshot(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_snapshot(engine_probe)
            }
            fn og_engine_probe_restore(&self, engine_probe: u32, id: String) -> bool {
                $crate::og_engine_probe_restore(engine_probe, id)
            }
            fn og_engine_probe_dump(&self, engine_probe: u32, desc: Option<String>) -> String {
                $crate::og_engine_probe_dump(engine_probe, desc)
            }
            fn og_engine_probe_commands(&self, engine_probe: u32, script: String) -> String {
                $crate::og_engine_probe_commands(engine_probe, script)
            }

            fn og_engine_probe_listen(&self, engine_probe: u32, seconds: Option<f64>) -> String {
                $crate::og_engine_probe_listen(engine_probe, seconds)
            }

            fn og_engine_probe_memory(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_memory(engine_probe)
            }

            fn og_engine_probe_describe(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_describe(engine_probe)
            }

            fn og_engine_probe_expect(&self, engine_probe: u32, what: String) -> String {
                $crate::og_engine_probe_expect(engine_probe, what)
            }

            fn og_engine_probe_expectations(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_expectations(engine_probe)
            }

            fn og_engine_probe_assertions(&self, engine_probe: u32, take: Option<bool>) -> String {
                $crate::og_engine_probe_assertions(engine_probe, take)
            }
            fn og_engine_probe_observe(
                &self,
                engine_probe: u32,
                records: Option<String>,
                types: Option<bool>,
            ) -> String {
                $crate::og_engine_probe_observe(engine_probe, records, types)
            }
            fn og_engine_probe_assertion_events(&self, engine_probe: u32) -> String {
                $crate::og_engine_probe_assertion_events(engine_probe)
            }

            fn og_engine_probe_save_scene(
                &self,
                engine_probe: u32,
                write: Option<bool>,
                spawned: Option<bool>,
            ) -> String {
                $crate::og_engine_probe_save_scene(engine_probe, write, spawned)
            }

            fn og_engine_probe_pick_ui(&self, engine_probe: u32, x: f64, y: f64) -> String {
                $crate::og_engine_probe_pick_ui(engine_probe, x, y)
            }
        }

        impl $crate::host_impl::api::DebugApi for $host {
            fn og_debug_line(&self, world: u32, from: String, to: String, desc: Option<String>) {
                $crate::og_debug_line(world, from, to, desc)
            }
            fn og_debug_box(&self, world: u32, center: String, size: String, desc: Option<String>) {
                $crate::og_debug_box(world, center, size, desc)
            }
            fn og_debug_sphere(
                &self,
                world: u32,
                center: String,
                radius: f64,
                desc: Option<String>,
            ) {
                $crate::og_debug_sphere(world, center, radius, desc)
            }
            fn og_debug_arrow(&self, world: u32, from: String, to: String, desc: Option<String>) {
                $crate::og_debug_arrow(world, from, to, desc)
            }
            fn og_debug_axes(&self, world: u32, position: String, size: f64, desc: Option<String>) {
                $crate::og_debug_axes(world, position, size, desc)
            }
            fn og_debug_text(
                &self,
                world: u32,
                position: String,
                text: String,
                desc: Option<String>,
            ) {
                $crate::og_debug_text(world, position, text, desc)
            }
            fn og_debug_clear(&self, world: u32) {
                $crate::og_debug_clear(world)
            }
            fn og_debug_set_view(&self, world: u32, view: String) {
                $crate::og_debug_set_view(world, view)
            }
            fn og_debug_view(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_debug_view(world))
            }
            fn og_debug_set_enabled(&self, world: u32, enabled: bool) {
                $crate::og_debug_set_enabled(world, enabled)
            }
            fn og_debug_enabled(&self, world: u32) -> bool {
                $crate::og_debug_enabled(world)
            }
        }

        impl $crate::host_impl::api::AssertApi for $host {
            fn og_assert_record(&self, world: u32, record: String) -> u32 {
                $crate::og_assert_record(world, record)
            }
        }
    };
}
