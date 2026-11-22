# rune-taffy

From rune (https://github.com/anomalyco/rune), commit
`01096f25021fb356b867c1b1868363467deb5143`, `crates/rune-taffy`, MIT (`LICENSE`).
`src/` is unmodified but for one change: `CompactStyle` keeps a non-zero
`order` (taffy's `CoreStyle::order`, patched into the vendored taffy) in its
cold `extra` style, and it and `FlexChildStyle` answer it. `src/calc.rs` (new) holds CSS `calc()`, `min()`, `max()` and `clamp()` expressions
for taffy's `calc` feature, which `Cargo.toml` turns on: `view.rs`'s
`resolve_calc_value` resolves a handle against its percentage basis instead of
returning 0. `Cargo.toml` writes out the fields rune's workspace
supplied (edition, licence, the taffy dependency's version) and drops its lints
and tests (Cargo.toml.orig is the original). og-ui's new core uses it as a
dependency, as published by rune.
