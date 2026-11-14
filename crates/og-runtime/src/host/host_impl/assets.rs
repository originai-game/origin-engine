//! The bindings of what a game loads and generates (LoadApi, StreamApi, ProcgenApi, SightApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_assets {
    ($host:ty) => {
        impl $crate::host_impl::api::LoadApi for $host {
            fn og_load_shader(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_shader(world, path))
            }
            fn og_load_vector(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_vector(world, path))
            }
            fn og_load_ui_material(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_ui_material(world, path))
            }
            fn og_load_post(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_post(world, path))
            }
            fn og_load_texture(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_texture(world, path))
            }
            fn og_load_gltf(&self, world: u32, path: String, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_load_gltf(world, path, desc))
            }
            fn og_load_scene(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_scene(world, path))
            }
            fn og_load_prefab(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_prefab(world, path))
            }
            fn og_load_status(&self, world: u32, asset: f64) -> String {
                $crate::og_load_status(world, asset)
            }
            fn og_load_gltf_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_load_gltf_info(world, asset))
            }
            fn og_load_bundle(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_bundle(world, path))
            }
            fn og_load_bundle_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_load_bundle_info(world, asset))
            }
            fn og_load_default_bundle(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_load_default_bundle(world))
            }
            fn og_load_font(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_font(world, path))
            }
            fn og_load_font_info(&self, world: u32, font: f64) -> String {
                $crate::host_impl::answer($crate::og_load_font_info(world, font))
            }
            fn og_load_animation(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_animation(world, path))
            }
            fn og_load_timeline(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_timeline(world, path))
            }
            fn og_load_vfx(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_vfx(world, path))
            }
            fn og_load_sound(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_sound(world, path))
            }
            fn og_load_sound_info(&self, world: u32, sound: f64) -> String {
                $crate::host_impl::answer($crate::og_load_sound_info(world, sound))
            }
            fn og_load_video(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_video(world, path))
            }
            fn og_load_video_info(&self, world: u32, video: f64) -> String {
                $crate::host_impl::answer($crate::og_load_video_info(world, video))
            }
            fn og_load_error(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::json($crate::og_load_error(world, asset))
            }
            fn og_load_lut(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_lut(world, path))
            }
            fn og_load_environment(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_environment(world, path))
            }
            fn og_load_environment_sun(&self, world: u32, map: f64) -> String {
                $crate::host_impl::answer($crate::og_load_environment_sun(world, map))
            }

            fn og_load_texture_array(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_texture_array(world, path))
            }
            fn og_load_emitter(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_emitter(world, path))
            }
            fn og_load_style(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_style(world, path))
            }
            fn og_load_document(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_document(world, path))
            }
            fn og_load_look(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_look(world, path))
            }
            fn og_load_look_info(&self, world: u32, look: f64) -> String {
                $crate::host_impl::answer($crate::og_load_look_info(world, look))
            }
            fn og_load_palette(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_palette(world, path))
            }
            fn og_load_flipbook(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_flipbook(world, path))
            }

            fn og_load_rig2d(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_rig2d(world, path))
            }

            fn og_load_image_data(
                &self,
                world: u32,
                name: String,
                bytes: Self::Bytes,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_load_image_data(
                    world,
                    name,
                    <Self as $crate::host_impl::HostWire>::bytes_in(bytes),
                    desc,
                ))
            }

            fn og_load_gltf_data(
                &self,
                world: u32,
                name: String,
                bytes: Self::Bytes,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_load_gltf_data(
                    world,
                    name,
                    <Self as $crate::host_impl::HostWire>::bytes_in(bytes),
                    desc,
                ))
            }

            fn og_load_environment_data(
                &self,
                world: u32,
                name: String,
                bytes: Self::Bytes,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_load_environment_data(
                    world,
                    name,
                    <Self as $crate::host_impl::HostWire>::bytes_in(bytes),
                    desc,
                ))
            }

            fn og_load_environment_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_load_environment_info(world, asset))
            }

            fn og_load_release(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_load_release(world, asset))
            }

            fn og_load_image_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_load_image_info(world, asset))
            }

            fn og_load_completions(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_load_completions(world))
            }

            fn og_load_replay_completions(&self, world: u32, completions: String) -> String {
                $crate::host_impl::answer($crate::og_load_replay_completions(world, completions))
            }

            fn og_load_rig(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_rig(world, path))
            }
            fn og_load_tilemap(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_load_tilemap(world, path))
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

        impl $crate::host_impl::api::SightApi for $host {
            fn og_sight_visible(&self, world: u32, team: u32, point: String) -> String {
                $crate::host_impl::answer($crate::og_sight_visible(world, team, point))
            }
            fn og_sight_explored(&self, world: u32, team: u32, point: String) -> String {
                $crate::host_impl::answer($crate::og_sight_explored(world, team, point))
            }
            fn og_sight_reveal(&self, world: u32, team: u32, point: String, radius: f64) -> String {
                $crate::host_impl::answer($crate::og_sight_reveal(world, team, point, radius))
            }
            fn og_sight_forget(&self, world: u32, team: u32) -> String {
                $crate::host_impl::answer($crate::og_sight_forget(world, team))
            }
        }

        impl $crate::host_impl::api::ProcgenApi for $host {
            fn og_procgen_noise(&self, desc: String, x: f64, y: f64, z: f64) -> f64 {
                $crate::og_procgen_noise(desc, x, y, z)
            }
            fn og_procgen_noise_grid(&self, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_procgen_noise_grid(desc))
            }
            fn og_procgen_poisson_disk(&self, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_procgen_poisson_disk(desc))
            }
            fn og_procgen_bsp(&self, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_procgen_bsp(desc))
            }
            fn og_procgen_wfc(&self, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_procgen_wfc(desc))
            }
        }
    };
}
