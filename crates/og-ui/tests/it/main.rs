#![feature(custom_test_frameworks, test)]
#![test_runner(gpu_tests::run)]
use og_render::wgpu as gpu_wgpu;
#[path = "../../../../tests/gpu.rs"]
mod gpu_tests;

#[path = "../a11y.rs"]
mod a11y;
#[path = "../button_contrast.rs"]
mod button_contrast;
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
#[path = "../hud_layout.rs"]
mod hud_layout;
#[path = "../kit_gfx.rs"]
mod kit_gfx;
#[path = "../layout.rs"]
mod layout;
#[path = "../linebreak.rs"]
mod linebreak;
#[path = "../motion.rs"]
mod motion;
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
#[path = "../style_contract.rs"]
mod style_contract;
#[path = "../support/mod.rs"]
mod support;
#[path = "../text.rs"]
mod text;
#[path = "../theme.rs"]
mod theme;
#[path = "../widgets.rs"]
mod widgets;
#[path = "../world.rs"]
mod world;
