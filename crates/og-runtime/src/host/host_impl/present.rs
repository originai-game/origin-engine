//! The bindings of what the player sees and hears (CameraApi, RenderApi, AtmosphereApi, TerrainApi, VfxApi, VideoApi, AudioApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_present {
    ($host:ty) => {
        impl $crate::host_impl::api::CameraApi for $host {
            fn og_camera_look_at(&self, world: u32, target: String) {
                $crate::og_camera_look_at(world, target)
            }
            fn og_camera_world_to_screen(&self, world: u32, point: String) -> String {
                $crate::host_impl::json($crate::og_camera_world_to_screen(world, point))
            }
            fn og_camera_screen_to_ray(&self, world: u32, x: f64, y: f64) -> String {
                $crate::host_impl::json($crate::og_camera_screen_to_ray(world, x, y))
            }
            fn og_camera_shake(&self, world: u32, trauma: f64) -> String {
                $crate::host_impl::answer($crate::og_camera_shake(world, trauma))
            }
            fn og_camera_punch(&self, world: u32, amount: f64) -> String {
                $crate::host_impl::answer($crate::og_camera_punch(world, amount))
            }
            fn og_camera_kick(&self, world: u32, direction: String, distance: f64) -> String {
                $crate::host_impl::answer($crate::og_camera_kick(world, direction, distance))
            }
        }

        impl $crate::host_impl::api::RenderApi for $host {
            fn og_render_shader_material(
                &self,
                world: u32,
                shader: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_shader_material(world, shader, desc))
            }
            fn og_render_set_material_shader(
                &self,
                world: u32,
                material: f64,
                shader: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_set_material_shader(
                    world, material, shader,
                ))
            }
            fn og_render_set_shader_param(
                &self,
                world: u32,
                material: f64,
                name: String,
                value: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_set_shader_param(
                    world, material, name, value,
                ))
            }
            fn og_render_box_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_box_geometry(world, desc))
            }
            fn og_render_plane_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_plane_geometry(world, desc))
            }
            fn og_render_sphere_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_sphere_geometry(world, desc))
            }
            fn og_render_buffer_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_buffer_geometry(world, desc))
            }
            fn og_render_update_geometry(
                &self,
                world: u32,
                geometry: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_update_geometry(world, geometry, desc))
            }
            fn og_render_geometry_info(&self, world: u32, geometry: f64) -> String {
                $crate::host_impl::answer($crate::og_render_geometry_info(world, geometry))
            }
            fn og_render_cylinder_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_cylinder_geometry(world, desc))
            }
            fn og_render_cone_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_cone_geometry(world, desc))
            }
            fn og_render_capsule_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_capsule_geometry(world, desc))
            }
            fn og_render_torus_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_torus_geometry(world, desc))
            }
            fn og_render_ring_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_ring_geometry(world, desc))
            }
            fn og_render_circle_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_circle_geometry(world, desc))
            }
            fn og_render_tube_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_tube_geometry(world, desc))
            }
            fn og_render_shape_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_shape_geometry(world, desc))
            }
            fn og_render_extrude_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_extrude_geometry(world, desc))
            }
            fn og_render_lathe_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_lathe_geometry(world, desc))
            }
            fn og_render_polyhedron_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_polyhedron_geometry(world, desc))
            }
            fn og_render_icosahedron_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_icosahedron_geometry(world, desc))
            }
            fn og_render_dodecahedron_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_dodecahedron_geometry(world, desc))
            }
            fn og_render_octahedron_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_octahedron_geometry(world, desc))
            }
            fn og_render_tetrahedron_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_tetrahedron_geometry(world, desc))
            }
            fn og_render_sweep_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_sweep_geometry(world, desc))
            }
            fn og_render_csg_geometry(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_csg_geometry(world, desc))
            }
            fn og_render_merge_geometries(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_merge_geometries(world, desc))
            }
            fn og_render_lines(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_lines(world, desc))
            }
            fn og_render_points(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_points(world, desc))
            }
            fn og_render_raycast(&self, world: u32, ray: String, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_raycast(world, ray, desc))
            }
            fn og_render_light2d_at(
                &self,
                world: u32,
                point: String,
                layers: Option<u32>,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_light2d_at(world, point, layers))
            }
            fn og_render_baked_lighting(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_render_baked_lighting(world))
            }
            fn og_render_sprite(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_sprite(world, desc))
            }
            fn og_render_measure_text(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_measure_text(world, desc))
            }
            fn og_render_standard_material(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_standard_material(world, desc))
            }
            fn og_render_update_material(
                &self,
                world: u32,
                material: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_update_material(world, material, desc))
            }
            fn og_render_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_render_capabilities(world))
            }
            fn og_render_render_target(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_render_target(world, desc))
            }
            fn og_render_set_post_param(
                &self,
                world: u32,
                post: f64,
                name: String,
                value: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_render_set_post_param(
                    world, post, name, value,
                ))
            }
            fn og_render_post_errors(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_render_post_errors(world))
            }
            fn og_render_apply_look(&self, world: u32, look: f64) -> String {
                $crate::host_impl::answer($crate::og_render_apply_look(world, look))
            }

            fn og_render_apply_quality(&self, world: u32, preset: String) -> String {
                $crate::host_impl::answer($crate::og_render_apply_quality(world, preset))
            }
            fn og_render_set_display(&self, world: u32, settings: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_set_display(world, settings))
            }

            fn og_render_reset_display(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_render_reset_display(world))
            }
            fn og_render_display(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_render_display(world))
            }
            fn og_render_look(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_render_look(world))
            }
            fn og_render_snap(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_render_snap(world, entity))
            }
            fn og_render_snap_camera(&self, world: u32) {
                $crate::og_render_snap_camera(world)
            }
            fn og_render_text(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_text(world, desc))
            }
            fn og_render_ribbon(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_ribbon(world, desc))
            }
            fn og_render_floating_text(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_render_floating_text(world, desc))
            }
        }

        impl $crate::host_impl::api::AtmosphereApi for $host {
            fn og_atmosphere_time_of_day(&self, world: u32) -> f64 {
                $crate::og_atmosphere_time_of_day(world)
            }
        }

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

        impl $crate::host_impl::api::VfxApi for $host {
            fn og_vfx_spawn(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_vfx_spawn(world, desc))
            }
            fn og_vfx_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_vfx_info(world, asset))
            }
        }

        impl $crate::host_impl::api::VideoApi for $host {
            fn og_video_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_video_capabilities(world))
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
    };
}
