//! Phase 2's modules' bindings (TerrainApi, VideoApi, StreamApi, AudioApi,
//! PathApi, TimelineApi, VfxApi, MotionApi, DestructionApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_p2 {
    ($host:ty) => {
        impl $crate::host_impl::api::TerrainApi for $host {
            fn og_terrain_load(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_terrain_load(world, path))
            }
            fn og_terrain_sample(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_terrain_sample(world, desc))
            }
            fn og_terrain_bounds(
                &self,
                world: u32,
                data: f64,
                origin: Option<String>,
                quaternion: Option<String>,
                scale: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_terrain_bounds(
                    world, data, origin, quaternion, scale,
                ))
            }
            fn og_terrain_edit(&self, world: u32, data: f64, brush: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_terrain_edit(world, data, brush))
            }
            fn og_terrain_stamp(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_terrain_stamp(world, desc))
            }
            fn og_terrain_unstamp(&self, world: u32, data: f64, stamp: u32) -> String {
                $crate::host_impl::answer($crate::og_terrain_unstamp(world, data, stamp))
            }
            fn og_terrain_layer(&self, world: u32, data: f64, index: u32) -> String {
                $crate::host_impl::answer($crate::og_terrain_layer(world, data, index))
            }
            fn og_terrain_set_layer(
                &self,
                world: u32,
                data: f64,
                index: u32,
                layer: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_terrain_set_layer(world, data, index, layer))
            }
        }

        impl $crate::host_impl::api::VideoApi for $host {
            fn og_video_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_video_capabilities(world))
            }
        }

        impl $crate::host_impl::api::StreamApi for $host {
            fn og_stream_status(&self, world: u32, group: String) -> String {
                $crate::host_impl::answer($crate::og_stream_status(world, group))
            }
            fn og_stream_request(&self, world: u32, group: String) -> String {
                $crate::host_impl::answer($crate::og_stream_request(world, group))
            }
            fn og_stream_release(&self, world: u32, group: String) -> String {
                $crate::host_impl::answer($crate::og_stream_release(world, group))
            }
            fn og_stream_budget(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_stream_budget(world))
            }
            fn og_stream_set_budget(&self, world: u32, bytes: f64) -> String {
                $crate::host_impl::answer($crate::og_stream_set_budget(world, bytes))
            }
        }

        impl $crate::host_impl::api::AudioApi for $host {
            fn og_audio_play_feedback(
                &self,
                world: u32,
                sound: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_audio_play_feedback(world, sound, desc))
            }
            fn og_audio_output_latency(&self, world: u32) -> f64 {
                $crate::og_audio_output_latency(world)
            }
            fn og_audio_create_sound(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_audio_create_sound(world, desc))
            }
        }

        impl $crate::host_impl::api::PathApi for $host {
            fn og_path_create(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_path_create(world, desc))
            }
            fn og_path_find(&self, world: u32, name: String) -> String {
                $crate::host_impl::answer($crate::og_path_find(world, name))
            }
            fn og_path_sample(&self, world: u32, path: f64, t: f64) -> String {
                $crate::host_impl::answer($crate::og_path_sample(world, path, t))
            }
            fn og_path_length(&self, world: u32, path: f64) -> String {
                $crate::host_impl::answer($crate::og_path_length(world, path))
            }
            fn og_path_nearest(&self, world: u32, path: f64, point: String) -> String {
                $crate::host_impl::answer($crate::og_path_nearest(world, path, point))
            }
        }

        impl $crate::host_impl::api::TimelineApi for $host {
            fn og_timeline_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_info(world, asset))
            }
            fn og_timeline_play(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_play(world, entity))
            }
            fn og_timeline_pause(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_pause(world, entity))
            }
            fn og_timeline_seek(&self, world: u32, entity: f64, time: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_seek(world, entity, time))
            }
            fn og_timeline_stop(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_stop(world, entity))
            }
        }

        impl $crate::host_impl::api::VfxApi for $host {
            fn og_vfx_spawn(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_vfx_spawn(world, desc))
            }
            fn og_vfx_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_vfx_info(world, asset))
            }
        }

        impl $crate::host_impl::api::MotionApi for $host {
            fn og_motion_pose(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_pose(world, entity, desc))
            }
            fn og_motion_place(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_place(world, entity, desc))
            }
            fn og_motion_face(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_face(world, entity, desc))
            }
            fn og_motion_walk_to(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_walk_to(world, entity, desc))
            }
            fn og_motion_jump(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_jump(world, entity, desc))
            }
            fn og_motion_clear(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_clear(world, entity))
            }
            fn og_motion_look_at(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_look_at(world, entity, desc))
            }
            fn og_motion_react(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_react(world, entity, desc))
            }
            fn og_motion_status(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_status(world, entity))
            }
            fn og_motion_capabilities(&self, world: u32) -> String {
                $crate::og_motion_capabilities(world)
            }
            fn og_motion_set_backend(&self, world: u32, backend: String) -> String {
                $crate::host_impl::answer($crate::og_motion_set_backend(world, backend))
            }
            fn og_motion_presentation(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_presentation(world, entity))
            }
            fn og_motion_poses(&self) -> String {
                $crate::og_motion_poses()
            }
        }

        impl $crate::host_impl::api::DestructionApi for $host {
            fn og_destruction_damage(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_damage(world, entity, desc))
            }
            fn og_destruction_explode(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_destruction_explode(world, desc))
            }
            fn og_destruction_fracture(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_destruction_fracture(world, entity))
            }
            fn og_destruction_info(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_destruction_info(world, entity))
            }
            fn og_destruction_material(&self, world: u32, kind: u32) -> String {
                $crate::host_impl::answer($crate::og_destruction_material(world, kind))
            }
            fn og_destruction_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_destruction_capabilities(world))
            }
            fn og_destruction_carve2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_carve2d(world, entity, desc))
            }
            fn og_destruction_fill2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_fill2d(world, entity, desc))
            }
            fn og_destruction_explode2d(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_destruction_explode2d(world, desc))
            }
            fn og_destruction_fracture2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_fracture2d(world, entity, desc))
            }
            fn og_destruction_cell2d(&self, world: u32, entity: f64, point: String) -> String {
                $crate::host_impl::answer($crate::og_destruction_cell2d(world, entity, point))
            }
        }
    };
}
