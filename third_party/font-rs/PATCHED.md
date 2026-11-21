# font-rs 0.1.3, patched

Upstream: <https://github.com/google/font-rs> (`font-rs` 0.1.3 as
published on crates.io, identical to upstream master 297f7490; Apache-2.0,
`LICENSE` here), used through the workspace's `[patch.crates-io]`. og-text
fills glyph coverage with it (`crates/og-text/src/raster.rs` is the
adapter). Only the crate's build files are kept: `src/`, `build.rs`,
`Cargo.toml`; its examples, benches and test font are left out.

One change, marked `og (PATCHED.md)` in place: `Raster::draw_quad`
flattens a quadratic into chords within 0.05 px of it (threshold
`devsq < 0.04`, `tol = 25`), where upstream allows 0.144 px (`0.333`,
`3`). At 12–16 px upstream's chords cut inside curves enough to draw a
glyph up to 3 % lighter than its outline (14 px `g`), and an edge pixel up
to 0.13 off its share, which a browser's FreeType fill does not.
