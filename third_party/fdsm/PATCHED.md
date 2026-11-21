# fdsm 0.8.0, patched

Upstream: <https://gitlab.com/Kyarei/fdsm> (`fdsm` 0.8.0 as published on
crates.io, MIT, `LICENSE` here from the upstream repository), used through
the workspace's `[patch.crates-io]`. The cook draws its MTSDF atlases with
it, and og-text its run-time distance fields for world, scaled and
effect text.

One change, marked `og (PATCHED.md)` in place: the cubic solver in
`src/bezier.rs` (`solve_cubic`) calls libm's `acos`, `sincos` and `cbrt`
instead of `f64`'s, and `Cargo.toml` adds `libm`. `f64`'s transcendental
functions are the host's (glibc on native hosts, Rust's own on wasm) and
may differ in the last bit, which would raster a distance field to
different bytes on the two hosts; the engine keeps both hosts' bytes
identical. Every other operation fdsm uses is IEEE arithmetic or `sqrt`,
correctly rounded everywhere. `src/visualize.rs` (feature `visualize`,
unused) keeps the host's `sin` and `cos`.
