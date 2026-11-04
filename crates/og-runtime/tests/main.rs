//! og-runtime's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.
//! Its own binary instead, declared in Cargo.toml: `extract_cost.rs`.

mod ao_resolution;
mod armed_emitters;
mod art_contract;
mod particle_events;
// It reads the frame recorder, which a publish build has not.
#[cfg(not(feature = "publish"))]
mod baked_scenes;
mod billboards;
mod camera_projection;
mod cells;
mod contact_hit;
mod cookie_cap;
mod crowd;
mod display;
mod drop_shadows;
mod garment;
mod geometry_uv_repeat;
mod insert_ranges;
#[cfg(not(feature = "publish"))]
mod inspector;
mod material_arrays;
mod mesh_flash;
mod mid_tick_save;
mod missing_glyphs;
mod model_clips;
mod model_lod;
mod model_reload;
mod motion_goals;
mod motion_performance;
mod nav_profiles;
mod no_renderer;
mod prefabs;
mod profile2d;
mod publish;
mod raycast_record_cost;
mod render_interpolation;
mod render_origin;
mod render_quality;
mod render_raycast;
mod restored_event;
mod review_views;
mod scene_string;
mod shadow_flags;
mod sky_off;
mod sprite_arrays;
mod stream_chunked;
mod stream_fetch;
mod stream_mips;
mod stream_release;
mod text_facing;
mod tilemap_reload;
mod tilemap_runtime;
mod time_groups;
mod timeline;
mod timeline_sound;
mod timeline_tracks;
mod tube_path;
mod ui_configure;
mod video_rendition;
mod virtual_cameras;
mod web_findings;
mod world_origin;
mod world_ui;
