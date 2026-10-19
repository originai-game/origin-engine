# Text glow: cost (owed row 18)

The skin's `textGlow` was four offset copies of the text under it (five
glyph instances per glyph, the copies' hard edges stacked). It is now one
halo from the font's distance field: og-render's MSDF overlay shader reads
the distance outside the ink (the MTSDF alpha, or the median for a plain
MSDF) and fades it quadratically to the glow's radius, capped where the
field ends. Each glyph is two instances, its glow then its fill, every
glyph's glow drawn before every fill.

Measured with `cargo test -p og-ui --test gfx -- --ignored text_glow_cost
--nocapture` (dev profile, whose wgpu is optimised), 2026-10-19, Apple M4
Pro, metal, with four other agents' builds and tests on the machine: 24
display-size (`displayMd`, 34 px) labels, "Wall 42", at 1280×1040, 600
frames per run, three runs each way alternating, best kept.

| | Glyph instances | ms per frame (best of 3) |
|---|---|---|
| no glow | 144 | 0.4952 |
| glow (distance field) | 288 | 0.5458 |
| difference | +144 (the old copies: +576) | +0.051 |

Runs: 0.5160, 0.5951, 0.4952 without; 0.6420, 0.5458, 0.5692 with. The
machine's load moves single runs by more than the difference; the glow's
fragment work is one texture sample and a few multiplies per pixel of a
glyph quad.

Reference frame: `crates/og-ui/references/metal/text-glow.png` (og-ui's
`tests/gfx.rs` `a_text_glow_is_a_halo_from_the_distance_field`: the solid
ink is unchanged by the glow under it, the background next to the ink is
tinted, nothing past the field's reach is). The neon and scifi packs, the
two with text glows, cook their glowing fonts with `px_range 12` (reach 6
atlas pixels instead of 2) and their sheets were re-recorded
(`rerecord/row17-18-kit-skin-{neon,scifi}{,-phone}.png`).
