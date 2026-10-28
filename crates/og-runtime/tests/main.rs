//! og-runtime's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.
//! Its own binary instead, declared in Cargo.toml: `extract_cost.rs`.

mod ao_resolution;
mod armed_emitters;
mod art_contract;
mod baked_scenes;
mod billboards;
mod camera_projection;
mod contact_hit;
mod cookie_cap;
mod drop_shadows;
mod geometry_uv_repeat;
mod material_arrays;
mod mesh_flash;
mod mid_tick_save;
mod missing_glyphs;
mod model_clips;
mod model_reload;
mod no_renderer;
mod profile2d;
mod raycast_record_cost;
mod render_quality;
mod render_raycast;
mod restored_event;
mod review_views;
mod scene_string;
mod shadow_flags;
mod sprite_arrays;
mod stream_chunked;
mod stream_fetch;
mod stream_release;
mod text_facing;
mod tilemap_reload;
mod timeline;
mod timeline_sound;
mod tube_path;
mod ui_configure;
mod web_findings;
