#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../antialias_default.rs"]
mod antialias_default;
#[path = "../ao_resolution.rs"]
mod ao_resolution;
#[path = "../armed_emitters.rs"]
mod armed_emitters;
#[path = "../art_contract.rs"]
mod art_contract;
#[path = "../particle_events.rs"]
mod particle_events;
// It reads the frame recorder, which a publish build has not.
#[cfg(not(feature = "publish"))]
#[path = "../baked_scenes.rs"]
mod baked_scenes;
#[path = "../billboards.rs"]
mod billboards;
#[path = "../camera_projection.rs"]
mod camera_projection;
#[path = "../cells.rs"]
mod cells;
#[path = "../contact_hit.rs"]
mod contact_hit;
#[path = "../cookie_cap.rs"]
mod cookie_cap;
#[path = "../crowd.rs"]
mod crowd;
#[cfg(feature = "destruction")]
#[path = "../destruction.rs"]
mod destruction;
#[path = "../display.rs"]
mod display;
#[path = "../drop_shadows.rs"]
mod drop_shadows;
#[path = "../extract_cost.rs"]
mod extract_cost;
#[path = "../font_packs.rs"]
mod font_packs;
#[path = "../garment.rs"]
mod garment;
#[path = "../geometry_uv_repeat.rs"]
mod geometry_uv_repeat;
#[path = "../insert_ranges.rs"]
mod insert_ranges;
#[cfg(not(feature = "publish"))]
#[path = "../inspector.rs"]
mod inspector;
#[path = "../instantiate_in_system.rs"]
mod instantiate_in_system;
#[path = "../invariants.rs"]
mod invariants;
#[path = "../material_arrays.rs"]
mod material_arrays;
#[path = "../mesh_flash.rs"]
mod mesh_flash;
#[path = "../mid_tick_save.rs"]
mod mid_tick_save;
#[path = "../missing_glyphs.rs"]
mod missing_glyphs;
#[path = "../model_clips.rs"]
mod model_clips;
#[path = "../model_lod.rs"]
mod model_lod;
#[path = "../model_reload.rs"]
mod model_reload;
#[path = "../motion_goals.rs"]
mod motion_goals;
#[path = "../motion_performance.rs"]
mod motion_performance;
#[cfg(feature = "nav")]
#[path = "../nav_profiles.rs"]
mod nav_profiles;
#[path = "../no_renderer.rs"]
mod no_renderer;
#[path = "../painted_2d.rs"]
mod painted_2d;
#[cfg(feature = "destruction")]
#[path = "../path_follower.rs"]
mod path_follower;
#[path = "../pixel_terrain.rs"]
mod pixel_terrain;
#[path = "../prefabs.rs"]
mod prefabs;
#[path = "../preview_overlay.rs"]
mod preview_overlay;
#[path = "../profile2d.rs"]
mod profile2d;
#[path = "../publish.rs"]
mod publish;
#[path = "../raycast_record_cost.rs"]
mod raycast_record_cost;
#[path = "../render_interpolation.rs"]
mod render_interpolation;
#[path = "../render_origin.rs"]
mod render_origin;
#[path = "../render_quality.rs"]
mod render_quality;
#[path = "../render_raycast.rs"]
mod render_raycast;
#[path = "../render_without.rs"]
mod render_without;
#[path = "../restored_event.rs"]
mod restored_event;
#[path = "../review_views.rs"]
mod review_views;
#[path = "../scatter_model_space.rs"]
mod scatter_model_space;
#[path = "../scene_string.rs"]
mod scene_string;
#[path = "../seabed.rs"]
mod seabed;
#[path = "../shadow_flags.rs"]
mod shadow_flags;
#[path = "../shared_models.rs"]
mod shared_models;
#[path = "../sky_off.rs"]
mod sky_off;
#[path = "../sprite_arrays.rs"]
mod sprite_arrays;
#[path = "../stream_chunked.rs"]
mod stream_chunked;
#[path = "../stream_fetch.rs"]
mod stream_fetch;
#[path = "../stream_mips.rs"]
mod stream_mips;
#[path = "../stream_release.rs"]
mod stream_release;
#[path = "../text_facing.rs"]
mod text_facing;
#[path = "../tilemap_reload.rs"]
mod tilemap_reload;
#[path = "../tilemap_runtime.rs"]
mod tilemap_runtime;
#[path = "../time_groups.rs"]
mod time_groups;
#[path = "../timeline.rs"]
mod timeline;
#[path = "../timeline_sound.rs"]
mod timeline_sound;
#[path = "../timeline_tracks.rs"]
mod timeline_tracks;
#[path = "../tube_path.rs"]
mod tube_path;
#[path = "../ui_configure.rs"]
mod ui_configure;
#[path = "../ui_material_reload.rs"]
mod ui_material_reload;
#[path = "../vfx.rs"]
mod vfx;
#[path = "../video_rendition.rs"]
mod video_rendition;
#[path = "../virtual_cameras.rs"]
mod virtual_cameras;
#[path = "../web_findings.rs"]
mod web_findings;
#[path = "../wire.rs"]
mod wire;
#[path = "../world_origin.rs"]
mod world_origin;
#[path = "../world_ui.rs"]
mod world_ui;
#[path = "../world_wind.rs"]
mod world_wind;
#[cfg(feature = "story")]
#[path = "../yarn.rs"]
mod yarn;
