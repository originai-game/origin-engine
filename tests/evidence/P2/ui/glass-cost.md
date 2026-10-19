# Glass: cost (owed row 33, the glass part)

A glass panel (og-render's `Overlay::glass`, og-ui's skin line
`glass <part> <blur> <fill> [tint]`) shows the scene under it blurred. A
frame with any glass renders its tonemap into a texture of its own, copies
it to the output (one fullscreen pass, `textureLoad`, exact), and
downsamples it into a six-level `Rgba16Float` pyramid (level k at 1/2^(k+1)
of the output, each a 13-tap blur of the level above); each panel then
samples its level, trilinear between levels. A frame without glass runs
none of this.

Measured with `cargo test -p og-ui --release --test gfx -- --ignored
glass_cost --nocapture`, 2026-10-19, Apple M4 Pro, metal, with four other
agents' builds and tests on the machine (load average 70 to 90): 1280×720,
fine stripes as the scene, four 280×400 glass panels (blur 16) against
none, 600 frames per run, three runs each way alternating, best kept.

| | ms per frame (best of 3) |
|---|---|
| no glass | 0.318 |
| four glass panels | 1.144 |
| difference | +0.83 |

Runs: 0.448, 0.318, 0.342 without; 2.424, 1.970, 1.144 with. The load moves
single runs by more than a millisecond; the fixed part is the copy and the
pyramid (about 1.33 times the output's pixels written once), the panels'
part one texture sample per covered pixel.

Reference frame: `crates/og-ui/references/metal/glass.png` (og-ui's
`tests/gfx.rs` `a_glass_panel_blurs_the_frame_under_it_and_nothing_else`:
every pixel outside the panel is the plain frame's, the stripes under it
blur to their average in linear light, the rounded corners stay sharp
stripes, and a full tint is the tint). The style-pack switch: the scifi
pack's dialogs and panels are glass (`kit-skin-scifi.png`,
`kit-skin-scifi-phone.png`, before and after in
`rerecord/row33-glass-*.png`).
