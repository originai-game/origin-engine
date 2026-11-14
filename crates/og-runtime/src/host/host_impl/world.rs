//! The world's bindings (ReflectApi, WorldApi, SystemApi, GameApi, TransformApi, RngApi, TimeApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_world {
    ($host:ty) => {
        impl $crate::host_impl::api::ReflectApi for $host {
            fn og_reflect_define_component(&self, name: String, fields: String) -> String {
                $crate::host_impl::answer($crate::og_reflect_define_component(name, fields))
            }
            fn og_reflect_define_resource(&self, name: String, fields: String) -> String {
                $crate::host_impl::answer($crate::og_reflect_define_resource(name, fields))
            }
            fn og_reflect_define_event(&self, name: String, fields: String) -> String {
                $crate::host_impl::answer($crate::og_reflect_define_event(name, fields))
            }
        }

        impl $crate::host_impl::api::WorldApi for $host {
            fn og_world_get_tick(&self, world: u32) -> u32 {
                $crate::og_world_get_tick(world)
            }
            fn og_world_spawn(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_spawn(world, desc))
            }
            fn og_world_despawn(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_world_despawn(world, entity))
            }
            fn og_world_alive(&self, world: u32, entity: f64) -> bool {
                $crate::og_world_alive(world, entity)
            }
            fn og_world_has(&self, world: u32, entity: f64, component: String) -> bool {
                $crate::og_world_has(world, entity, component)
            }
            fn og_world_insert(
                &self,
                world: u32,
                entity: f64,
                component: String,
                value: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_world_insert(world, entity, component, value))
            }
            fn og_world_remove(&self, world: u32, entity: f64, component: String) -> String {
                $crate::host_impl::answer($crate::og_world_remove(world, entity, component))
            }
            fn og_world_get(&self, world: u32, entity: f64, component: String) -> String {
                $crate::host_impl::answer($crate::og_world_get(world, entity, component))
            }
            fn og_world_set(
                &self,
                world: u32,
                entity: f64,
                component: String,
                value: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_world_set(world, entity, component, value))
            }
            fn og_world_resource(&self, world: u32, resource: String) -> String {
                $crate::host_impl::answer($crate::og_world_resource(world, resource))
            }
            fn og_world_find(&self, world: u32, name: String) -> String {
                $crate::host_impl::json($crate::og_world_find(world, name))
            }
            fn og_spatial_index_get_count(&self, spatial_index: u32) -> u32 {
                $crate::og_spatial_index_get_count(spatial_index)
            }
            fn og_spatial_index_within(
                &self,
                spatial_index: u32,
                center: String,
                radius: f64,
            ) -> String {
                $crate::host_impl::json($crate::og_spatial_index_within(
                    spatial_index,
                    center,
                    radius,
                ))
            }
            fn og_spatial_index_nearest(
                &self,
                spatial_index: u32,
                center: String,
                count: u32,
            ) -> String {
                $crate::host_impl::json($crate::og_spatial_index_nearest(
                    spatial_index,
                    center,
                    count,
                ))
            }
            fn og_world_children(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::json($crate::og_world_children(world, entity))
            }
            fn og_world_removed(&self, world: u32, component: String) -> String {
                $crate::host_impl::answer($crate::og_world_removed(world, component))
            }
            fn og_world_spatial(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_spatial(world, desc))
            }
            fn og_world_state(&self, world: u32, name: String) -> String {
                $crate::host_impl::json($crate::og_world_state(world, name))
            }
            fn og_world_set_state(&self, world: u32, name: String, value: String) -> String {
                $crate::host_impl::answer($crate::og_world_set_state(world, name, value))
            }
            fn og_world_prefab_of(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::json($crate::og_world_prefab_of(world, entity))
            }
            fn og_world_name_of(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::json($crate::og_world_name_of(world, entity))
            }
            fn og_world_query(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_query(world, desc))
            }
            fn og_world_events(&self, world: u32, event: String) -> String {
                $crate::host_impl::json($crate::og_world_events(world, event))
            }
            fn og_world_emit(&self, world: u32, event: String, payload: String) -> String {
                $crate::host_impl::answer($crate::og_world_emit(world, event, payload))
            }
            fn og_world_instantiate(&self, world: u32, asset: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_instantiate(world, asset, desc))
            }
            fn og_world_snapshot(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_snapshot(world, desc))
            }
            fn og_world_restore(&self, world: u32, save: String, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_world_restore(world, save, desc))
            }
            fn og_world_open_save(&self, world: u32, save: String) -> String {
                $crate::host_impl::answer($crate::og_world_open_save(world, save))
            }
            fn og_save_edit_get_version(&self, save_edit: u32) -> u32 {
                $crate::og_save_edit_get_version(save_edit)
            }
            fn og_save_edit_types(&self, save_edit: u32) -> String {
                $crate::host_impl::answer($crate::og_save_edit_types(save_edit))
            }
            fn og_save_edit_records(&self, save_edit: u32, name: String) -> String {
                $crate::host_impl::answer($crate::og_save_edit_records(save_edit, name))
            }
            fn og_save_edit_set_records(
                &self,
                save_edit: u32,
                name: String,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_save_edit_set_records(save_edit, name, desc))
            }
            fn og_save_edit_rename_type(&self, save_edit: u32, from: String, to: String) -> String {
                $crate::host_impl::answer($crate::og_save_edit_rename_type(save_edit, from, to))
            }
            fn og_save_edit_rename_field(
                &self,
                save_edit: u32,
                name: String,
                from: String,
                to: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_save_edit_rename_field(
                    save_edit, name, from, to,
                ))
            }
            fn og_save_edit_drop_type(&self, save_edit: u32, name: String) -> String {
                $crate::host_impl::answer($crate::og_save_edit_drop_type(save_edit, name))
            }
            fn og_save_edit_spawn(&self, save_edit: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_save_edit_spawn(save_edit, desc))
            }
            fn og_save_edit_text(&self, save_edit: u32) -> String {
                $crate::host_impl::answer($crate::og_save_edit_text(save_edit))
            }
            fn og_save_edit_close(&self, save_edit: u32) {
                $crate::og_save_edit_close(save_edit)
            }
            fn og_query_count(&self, query: u32) -> u32 {
                $crate::og_query_count(query)
            }
            fn og_archetype_view_get_count(&self, archetype_view: u32) -> u32 {
                $crate::og_archetype_view_get_count(archetype_view)
            }
            fn og_archetype_view_get_tick(&self, archetype_view: u32) -> u32 {
                $crate::og_archetype_view_get_tick(archetype_view)
            }
            fn og_archetype_view_get_entities(&self, archetype_view: u32) -> Self::Array {
                <Self as $crate::host_impl::HostWire>::array($crate::view_entities(archetype_view))
            }
            fn og_archetype_view_get_rows(&self, archetype_view: u32) -> Self::Array {
                <Self as $crate::host_impl::HostWire>::array($crate::view_rows(archetype_view))
            }
            fn og_archetype_view_entity(&self, archetype_view: u32, index: u32) -> f64 {
                $crate::og_archetype_view_entity(archetype_view, index)
            }
        }

        impl $crate::host_impl::api::SystemApi for $host {
            fn og_system_get_name(&self, system: u32) -> String {
                $crate::og_system_get_name(system)
            }
            fn og_system_get_reads(&self, system: u32) -> String {
                $crate::host_impl::json($crate::og_system_get_reads(system))
            }
            fn og_system_get_writes(&self, system: u32) -> String {
                $crate::host_impl::json($crate::og_system_get_writes(system))
            }
            fn og_hook_get_name(&self, hook: u32) -> String {
                $crate::og_hook_get_name(hook)
            }
            fn og_system_define_hook(
                &self,
                name: String,
                component: String,
                desc: String,
                run: u32,
            ) -> u32 {
                $crate::og_system_define_hook(name, component, desc, run)
            }
            fn og_system_define_system(&self, name: String, desc: String, run: u32) -> u32 {
                $crate::og_system_define_system(name, desc, run)
            }
        }

        impl $crate::host_impl::api::GameApi for $host {
            fn og_game_get_tick(&self, game: u32) -> u32 {
                $crate::og_game_get_tick(game)
            }
            fn og_game_get_systems(&self, game: u32) -> String {
                $crate::host_impl::json($crate::og_game_get_systems(game))
            }
            fn og_game_game(&self, desc: Option<String>) -> u32 {
                $crate::og_game_game(desc)
            }
        }

        impl $crate::host_impl::api::TransformApi for $host {
            fn og_transform_compose(
                &self,
                position: String,
                quaternion: String,
                scale: String,
            ) -> String {
                $crate::host_impl::json($crate::og_transform_compose(position, quaternion, scale))
            }
            fn og_transform_decompose(&self, matrix: String) -> String {
                $crate::host_impl::json($crate::og_transform_decompose(matrix))
            }
            fn og_transform_look_at(
                &self,
                eye: String,
                target: String,
                up: Option<String>,
            ) -> String {
                $crate::host_impl::json($crate::og_transform_look_at(eye, target, up))
            }
            fn og_transform_face(
                &self,
                position: String,
                target: String,
                up: Option<String>,
            ) -> String {
                $crate::host_impl::json($crate::og_transform_face(position, target, up))
            }
            fn og_transform_forward(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_forward(quaternion))
            }
            fn og_transform_right(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_right(quaternion))
            }
            fn og_transform_up(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_up(quaternion))
            }
            fn og_transform_camera_forward(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_camera_forward(quaternion))
            }
            fn og_transform_camera_right(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_camera_right(quaternion))
            }
            fn og_transform_heading(&self, quaternion: String) -> f64 {
                $crate::og_transform_heading(quaternion)
            }
            fn og_transform_from_heading(&self, heading: f64) -> String {
                $crate::host_impl::json($crate::og_transform_from_heading(heading))
            }
            fn og_transform_face2d(&self, position: String, target: String) -> String {
                $crate::host_impl::json($crate::og_transform_face2d(position, target))
            }
            fn og_transform_forward2d(&self, quaternion: String) -> String {
                $crate::host_impl::json($crate::og_transform_forward2d(quaternion))
            }
            fn og_transform_heading2d(&self, quaternion: String) -> f64 {
                $crate::og_transform_heading2d(quaternion)
            }
            fn og_transform_from_heading2d(&self, heading: f64) -> String {
                $crate::host_impl::json($crate::og_transform_from_heading2d(heading))
            }
            fn og_transform_from_axis_angle(&self, axis: String, angle: f64) -> String {
                $crate::host_impl::json($crate::og_transform_from_axis_angle(axis, angle))
            }
        }

        impl $crate::host_impl::api::RngApi for $host {
            fn og_rng_stream_get_seed(&self, rng_stream: u32) -> u32 {
                $crate::og_rng_stream_get_seed(rng_stream)
            }
            fn og_rng_stream_next(&self, rng_stream: u32) -> f64 {
                $crate::og_rng_stream_next(rng_stream)
            }
            fn og_rng_stream_range(&self, rng_stream: u32, low: f64, high: f64) -> f64 {
                $crate::og_rng_stream_range(rng_stream, low, high)
            }
            fn og_rng_stream_integer(&self, rng_stream: u32, low: i32, high: i32) -> i32 {
                $crate::og_rng_stream_integer(rng_stream, low, high)
            }
            fn og_rng_stream_chance(&self, rng_stream: u32, p: f64) -> bool {
                $crate::og_rng_stream_chance(rng_stream, p)
            }
            fn og_rng_stream_fork(&self, rng_stream: u32, label: String) -> u32 {
                $crate::og_rng_stream_fork(rng_stream, label)
            }
            fn og_rng_next(&self, world: u32) -> f64 {
                $crate::og_rng_next(world)
            }
            fn og_rng_range(&self, world: u32, low: f64, high: f64) -> f64 {
                $crate::og_rng_range(world, low, high)
            }
            fn og_rng_integer(&self, world: u32, low: i32, high: i32) -> i32 {
                $crate::og_rng_integer(world, low, high)
            }
            fn og_rng_chance(&self, world: u32, p: f64) -> bool {
                $crate::og_rng_chance(world, p)
            }
            fn og_rng_fork(&self, world: u32, label: String) -> u32 {
                $crate::og_rng_fork(world, label)
            }
            fn og_rng_stream_weighted(&self, rng_stream: u32, weights: String) -> i32 {
                $crate::og_rng_stream_weighted(rng_stream, weights)
            }
            fn og_rng_stream_permutation(&self, rng_stream: u32, n: u32) -> String {
                $crate::host_impl::json($crate::og_rng_stream_permutation(rng_stream, n))
            }
            fn og_rng_weighted(&self, world: u32, weights: String) -> i32 {
                $crate::og_rng_weighted(world, weights)
            }
            fn og_rng_permutation(&self, world: u32, n: u32) -> String {
                $crate::host_impl::json($crate::og_rng_permutation(world, n))
            }
        }

        impl $crate::host_impl::api::TimeApi for $host {
            fn og_time_set_scale(&self, world: u32, scale: f64) -> String {
                $crate::host_impl::answer($crate::og_time_set_scale(world, scale))
            }
            fn og_time_hit_stop(&self, world: u32, ticks: u32) -> String {
                $crate::host_impl::answer($crate::og_time_hit_stop(world, ticks))
            }
            fn og_time_set_group_scale(&self, world: u32, group: u32, scale: f64) -> String {
                $crate::host_impl::answer($crate::og_time_set_group_scale(world, group, scale))
            }
            fn og_time_group_dt(&self, world: u32, entity: f64) -> f64 {
                $crate::og_time_group_dt(world, entity)
            }
        }
    };
}
