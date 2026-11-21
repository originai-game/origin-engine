# tiny-skia 0.12.0, patched

Upstream: <https://github.com/linebender/tiny-skia> (`tiny-skia` 0.12.0 as
published on crates.io, BSD-3-Clause, `LICENSE` here), used through the
workspace's `[patch.crates-io]`. resvg draws SVG with it, and og-text
paints colour (COLR) glyphs with it.

One change, marked `og (PATCHED.md)` in place: `f32x4::recip_fast` and
`f32x8::recip_fast` (`src/wide/`) divide exactly (`splat(1.0) / self`) on
every target. Upstream uses SSE's `_mm_rcp_ps`, AVX's `_mm256_rcp_ps` and
NEON's estimate, whose precision the CPU chooses, while wasm divides
exactly, so a pixel the colour-dodge and colour-burn blend modes reach
(their only callers, `src/pipeline/highp.rs`) would differ between the
native and wasm hosts. The engine keeps both hosts' bytes identical. Every
other operation in the pipelines is IEEE arithmetic, `sqrt` or integer
maths, the same on every target.
