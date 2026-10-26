# Debt: the debug-overlay `og.debug.text` label

Branch `debt/debug-overlay-text`, from `30376326`. The item owed by the
depth wave's Metal references (`707d7f7d`, docs/phase-1.md): "the
`debug-overlay` text label is not visible in either tier".

## Outcome: already visible, nothing to fix

The label was missing because the scene never drew one. The scene pushed
no text at all, so no tier could show it. That was fixed before this
stream, after `707d7f7d`:

- `93c52aaf` (merge of `bf43876e`, `debt/geometry-video`): the scene draws
  two labels, "sphere 1.2 m" above the sphere and "hidden box" at the box
  behind the wall. They take og-runtime's `og.debug.text` route: the
  world point is projected through the camera, the label is centred above
  it, and it is drawn in the overlay's UI layer.
- `b80aa70c`: the Metal reference re-recorded with both labels.

Neither og-render nor og-runtime has a defect here. No engine or scene
change, no new test and no re-recorded frame came out of this stream.
**No Metal frame is owed.**

## Frames read at 30376326

**og-gfx on this orb's lavapipe** (dev profile,
`VK_ICD_FILENAMES=…/lvp_icd.json target/debug/og-gfx debug-overlay`):
`tier=vulkan-cpu`, `glyphs=19`. That is the non-space characters of both
labels (10 + 9).

- `frames/debug-overlay-vulkan-cpu.png`
  - "sphere 1.2 m" is in green above the green debug box, over the bloom,
    around x 390–520, y 228–247.
  - "hidden box" is in magenta above the magenta box, around x 765–878,
    y 260–280. It sits across the wall's top edge and is drawn whole over
    both the wall and the floor (`frames/debug-overlay-hidden-box-x6.png`,
    6× crop).
  - The rest of the frame is as before: box, sphere circles, arrow,
    axes, and the box seen through the wall.
- `crates/og-tests/references/metal/debug-overlay.png` (the committed
  Metal reference) shows the same two labels at the same places. Its
  per-pixel difference from the lavapipe frame averages 0.12/255.

**Native og host**: `og run templates/fx-showcase --headless --size 1280x720
--out frames --commands "0.5:station:geometry;1.50:screenshot:station-geometry-1.50.png;2.00:screenshot:station-geometry-2.00.png"`
(dev `target/debug/og`, lavapipe). The station calls
`og.debug.text(world, probe, "<distance> m", { color: pink })` every
tick.

- `frames/station-geometry-1.50.png`: "3.52 m" in pale pink at about
  (768, 102), above the top of the probe's pink arrow.
- `frames/station-geometry-2.00.png`: "3.40 m" at about (656, 105). The
  label follows the probe. This matches the Metal reference
  `templates/fx-showcase/references/metal/geometry.png` (also "3.40 m",
  same place).

## Not done (by the stream's rules)

- No gate, `og test`, web-reference or reference capture.
- Only `cargo build -p og-tests --bin og-gfx` and `cargo build -p og-cli
  --bin og` (dev), to capture the frames above.
- The labels are guarded only by the whole-frame comparison.
  `references/vulkan-cpu/` has no frame of the scene, so `gfx::check`
  judges lavapipe against the Metal frame, which now carries the labels.
  No region read looks at them specifically. Adding one would be a new
  check, not a fix. It was left out because the brief asked to stop once
  the label is visible.
