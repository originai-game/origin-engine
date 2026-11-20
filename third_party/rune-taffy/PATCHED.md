# rune-taffy

From rune (https://github.com/anomalyco/rune), commit
`01096f25021fb356b867c1b1868363467deb5143`, `crates/rune-taffy`, MIT (`LICENSE`).
`src/` is unmodified. `Cargo.toml` writes out the fields rune's workspace
supplied (edition, licence, the taffy dependency's version) and drops its lints
and tests (Cargo.toml.orig is the original). og-ui's new core uses it as a
dependency, as published by rune.
