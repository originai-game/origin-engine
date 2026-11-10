#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../atmosphere.rs"]
mod atmosphere;
#[path = "../cluster_compute.rs"]
mod cluster_compute;
#[path = "../culling.rs"]
mod culling;
#[path = "../debug_views.rs"]
mod debug_views;
#[path = "../effects.rs"]
mod effects;
#[path = "../exposure.rs"]
mod exposure;
#[path = "../fluids3d.rs"]
mod fluids3d;
#[path = "../fork_fields.rs"]
mod fork_fields;
#[path = "../fsr.rs"]
mod fsr;
#[path = "../geometry.rs"]
mod geometry;
#[path = "../gfx.rs"]
mod gfx;
#[path = "../graph.rs"]
mod graph;
#[path = "../hooks.rs"]
mod hooks;
#[path = "../light.rs"]
mod light;
#[path = "../lights.rs"]
mod lights;
#[path = "../materials.rs"]
mod materials;
#[path = "../moon.rs"]
mod moon;
#[path = "../ocean.rs"]
mod ocean;
#[path = "../perf.rs"]
mod perf;
#[path = "../physical.rs"]
mod physical;
#[path = "../presentation.rs"]
mod presentation;
#[path = "../probes.rs"]
mod probes;
#[path = "../removal.rs"]
mod removal;
#[path = "../render.rs"]
mod render;
#[path = "../render_origin.rs"]
mod render_origin;
#[path = "../render_without.rs"]
mod render_without;
#[path = "../shadows.rs"]
mod shadows;
#[path = "../sprites2d.rs"]
mod sprites2d;
#[path = "../stylise_line_toon_paint.rs"]
mod stylise_line_toon_paint;
#[path = "../stylise_print.rs"]
mod stylise_print;
#[path = "../stylise_space.rs"]
mod stylise_space;
#[path = "../taa_look.rs"]
mod taa_look;
#[path = "../taau.rs"]
mod taau;
#[path = "../targets.rs"]
mod targets;
#[path = "../terrain.rs"]
mod terrain;
#[path = "../ui.rs"]
mod ui;
#[path = "../upload.rs"]
mod upload;
#[path = "../visual_bodies.rs"]
mod visual_bodies;
#[path = "../wasm.rs"]
mod wasm;
#[path = "../weather_surfaces.rs"]
mod weather_surfaces;
