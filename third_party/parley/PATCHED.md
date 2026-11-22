# Parley 0.12.0, patched

Upstream: <https://github.com/linebender/parley>, Apache-2.0 OR MIT.
The source and normalized Cargo.toml come from the 0.12.0 crate.

`Layout::set_discretionary_breaks` accepts source byte offsets and the
shaped advances of discretionary endings. Line breaking records a candidate
only if its ending fits, and adds the advance only when taking that break.
Min-content measurement includes the ending; max-content does not.
`Line::discretionary_offset` locates the ending beside the logically last
run after bidi reordering. Positioned runs leave its gap on that run's
visual end. The caller shapes and draws the ending, without inserting
characters into the source or changing cursor and selection byte offsets.

`RangedBuilder::set_shaping_boundaries` splits shaping at source byte offsets.
This lets a narrow line take an opportunity inside a ligature without
globally disabling ligatures or changing source offsets. og-text restores
the original shaping on wide reflow and measures min-content separately.

Changes are in `layout/{data,layout,line_break,line}.rs`, `builder.rs` and
`shape/mod.rs`. Other source files are unchanged upstream. og-text tests
exercise the public layout and glyph contracts, including a hyphen that
would overflow, ligature breaks, wide reflow, and bidi.
