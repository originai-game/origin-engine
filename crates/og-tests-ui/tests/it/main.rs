#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../a11y.rs"]
mod a11y;
#[path = "../box_decor.rs"]
mod box_decor;
#[path = "../color_vision.rs"]
mod color_vision;
#[path = "../css_effects.rs"]
mod css_effects;
#[path = "../css_filters.rs"]
mod css_filters;
#[path = "../css_paint.rs"]
mod css_paint;
#[path = "../dressing.rs"]
mod dressing;
#[path = "../gfx.rs"]
mod gfx;
#[path = "../glyphs.rs"]
mod glyphs;
#[path = "../html_controls.rs"]
mod html_controls;
#[path = "../html_text.rs"]
mod html_text;
#[path = "../inline_svg.rs"]
mod inline_svg;
#[path = "../input_types.rs"]
mod input_types;
#[path = "../kit_gfx.rs"]
mod kit_gfx;
#[path = "../mounted_layers.rs"]
mod mounted_layers;
#[path = "../screen_layers.rs"]
mod screen_layers;
#[path = "../support/mod.rs"]
mod support;
#[path = "../svg_defs.rs"]
mod svg_defs;
#[path = "../svg_text.rs"]
mod svg_text;
#[path = "../text_path.rs"]
mod text_path;
#[path = "../touch_controls.rs"]
mod touch_controls;
#[path = "../ui_debug.rs"]
mod ui_debug;
#[path = "../user_select.rs"]
mod user_select;
