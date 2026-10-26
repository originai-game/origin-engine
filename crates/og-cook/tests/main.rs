//! og-cook's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.

mod anim_compress;
mod animation;
mod behaviour;
mod compressed;
mod cook_report;
mod environment;
mod flipbook;
mod fx;
mod lod;
mod look_builtins;
mod model_text;
mod physical;
mod raycast_bvh;
mod retarget;
mod rig2d;
mod scene_bool;
mod scene_list;
mod scene_string;
mod scene_write_back;
mod sfx;
mod shader;
mod sound_chunks;
mod stream;
mod strings;
mod support;
mod template;
mod terrain;
mod texture_cache;
mod textures_p4;
mod tilemap;
mod timeline;
mod vox;
