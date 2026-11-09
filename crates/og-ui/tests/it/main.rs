#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../a11y.rs"]
mod a11y;
#[path = "../box_decor.rs"]
mod box_decor;
#[path = "../button_contrast.rs"]
mod button_contrast;
#[path = "../color_vision.rs"]
mod color_vision;
#[path = "../component_documents.rs"]
mod component_documents;
#[path = "../control_props.rs"]
mod control_props;
#[path = "../css_contain.rs"]
mod css_contain;
#[path = "../css_effects.rs"]
mod css_effects;
#[path = "../css_filters.rs"]
mod css_filters;
#[path = "../css_interact.rs"]
mod css_interact;
#[path = "../css_layout.rs"]
mod css_layout;
#[path = "../css_paint.rs"]
mod css_paint;
#[path = "../custom_properties.rs"]
mod custom_properties;
#[path = "../documents.rs"]
mod documents;
#[path = "../dressing.rs"]
mod dressing;
#[path = "../features.rs"]
mod features;
#[path = "../fields.rs"]
mod fields;
#[path = "../gfx.rs"]
mod gfx;
#[path = "../glyphs.rs"]
mod glyphs;
#[path = "../html_controls.rs"]
mod html_controls;
#[path = "../html_text.rs"]
mod html_text;
#[path = "../hud_layout.rs"]
mod hud_layout;
#[path = "../inline_svg.rs"]
mod inline_svg;
#[path = "../input_types.rs"]
mod input_types;
#[path = "../kit_gfx.rs"]
mod kit_gfx;
#[path = "../layout.rs"]
mod layout;
#[path = "../linebreak.rs"]
mod linebreak;
#[path = "../motion.rs"]
mod motion;
#[path = "../mounted_layers.rs"]
mod mounted_layers;
#[path = "../perf.rs"]
mod perf;
#[path = "../probe.rs"]
mod probe;
#[path = "../recognizers.rs"]
mod recognizers;
#[path = "../rich.rs"]
mod rich;
#[path = "../rtl.rs"]
mod rtl;
#[path = "../runs.rs"]
mod runs;
#[path = "../screen_layers.rs"]
mod screen_layers;
#[path = "../slots.rs"]
mod slots;
#[path = "../style.rs"]
mod style;
#[path = "../style_pack.rs"]
mod style_pack;
#[path = "../support/mod.rs"]
mod support;
#[path = "../svg_defs.rs"]
mod svg_defs;
#[path = "../svg_text.rs"]
mod svg_text;
#[path = "../text.rs"]
mod text;
#[path = "../text_path.rs"]
mod text_path;
#[path = "../theme.rs"]
mod theme;
#[path = "../touch_controls.rs"]
mod touch_controls;
#[path = "../ui_debug.rs"]
mod ui_debug;
#[path = "../user_select.rs"]
mod user_select;
#[path = "../widgets.rs"]
mod widgets;
#[path = "../world.rs"]
mod world;
