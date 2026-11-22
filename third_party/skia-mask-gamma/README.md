# Skia's mask gamma

Skia's text preblend (`SkTMaskGamma_build_correcting_lut`,
`SkColorSpaceLuminance::computeLuminance`, `SkTMaskGamma::CanonicalColor`
and `sk_t_scale255`, from https://github.com/google/skia,
`src/core/SkMaskGamma.cpp` and `src/core/SkMaskGamma.h` at revision
`d2b9e48baf1697760afc1dc8ea3ad40110b8cacc`), ported to
`crates/og-render/src/systems/ui/paint.rs` (`preblend_table`), whose table
`crates/og-render/shaders/ui_paint.wgsl` applies to a glyph's coverage.
The constants are Chrome on Linux's (`SK_GAMMA_EXPONENT` 1.2,
`SK_GAMMA_CONTRAST` 0.2), with Skia's three luminance bits. BSD-3-Clause,
Copyright (c) 2011 Google Inc., the notice in `LICENSE`.
