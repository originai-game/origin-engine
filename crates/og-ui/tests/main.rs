//! og-ui's integration tests, one binary: each file under tests/ is a
//! module of it, so an edit relinks one test executable instead of one
//! per file (docs/ci.md §Test binaries). nextest still runs every test
//! in a process of its own. A new test file is a `mod` line here.

mod a11y;
mod button_contrast;
mod fields;
mod gfx;
mod glyphs;
mod kit_gfx;
mod layout;
mod linebreak;
mod materials_gfx;
mod pack_contrast;
mod packs;
mod probe;
mod rich;
mod rtl;
mod runs;
mod screen_layers;
mod style;
mod support;
mod text;
mod text_gfx;
mod widgets;
