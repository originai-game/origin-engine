# font-rs

`crates/og-ui/src/colr.rs` ports the accumulation rasterizer of
[font-rs](https://github.com/raphlinus/font-rs) (`src/raster.rs`, Raph
Levien, Copyright 2015 Google Inc.), Apache-2.0 (`LICENSE`): each edge adds
its exact signed area to the cells it crosses and a row's running sum is the
coverage. The port works in f64 over a padded row stride, clamps edges to the
grid's sides instead of requiring them inside, and keeps non-zero coverage as
the absolute sum, capped at one. It draws COLR colour glyphs at cook time and, for a colour font that ships its file, at run time.
