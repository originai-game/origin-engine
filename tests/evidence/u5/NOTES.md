# U5: world-space UI sample (docs/ui.md §14, §5.5)

`sample/` is an arena game with these parts:

- 100 pillars, each with a nameplate: an anchored element holding a name and a health bar. Its settings are `declutter: "stack"`, priority by row (front rows rank higher), `occlusion: "fade"`, LOD past 22 m (the name alone) and `maxDistance: 45`.
- Damage numbers: a mass element template spawned 34 a tick with a 60-tick life, about 2,070 live.
- An "Extraction" waypoint behind the camera, edge-clamped with an arrow and its distance.
- A terminal world panel on a plane whose "Toggle gate" button takes a press through the pointer's ray.
- A loadout menu with a character viewport (an `og.CameraView` of a turntable on layer 1) that turns on a drag, and a Claim button that throws confetti (`og.ui.particles`).
- A pulsing low-health vignette (`og.ui.meta`).

Nothing was drawn in the orb (the owner's rule, AGENTS.md d4fdb7ae: no lavapipe). The frames below are to be drawn on macrunner (`metal`). What the orb checked is a logic run with `OG_RENDERER=none` and the rebuilt dev addon. It used the same commands without screenshots, then `get:arena:Arena` and `dump_ui`.

- The press at 277,447 reached the terminal button: `presses 1`, and the label reads "Gate: OPEN".
- The click on Claim registered: `claims 1`.
- The drag in the viewport turned the figure by 1.2 rad more than the idle turn.
- The waypoint was clamped at the bottom edge (pivot y 626 at 1280×720), labelled "20 m".
- At tick 120, 73 of the 100 nameplates were visible, with no two of them overlapping. The other 27 were hidden by declutter because they could not stack within reach.
- The HUD read "Nameplates 73 / 100 · Damage numbers 2074".

## Commands (seconds, not ticks)

```sh
og run tests/evidence/u5/sample --headless --size 1280x720 --out <dir> --commands \
  "2:screenshot:arena.png;\
2.1:touch_move:277,447;2.15:touch_start;2.2:touch_end;2.5:screenshot:terminal-pressed.png;\
3:touch_move_to:Claim;3.05:touch_start;3.1:touch_end;3.3:screenshot:confetti.png;\
3.5:touch_move:1088,250;3.55:touch_start;3.6:touch_move:1188,250;3.65:touch_end;3.8:screenshot:viewport-turned.png"
```

## What each frame should show

- `arena.png` (tick 120):
  - The camera looks down over ten rows of stone pillars on a grey floor, with a wall across the back left.
  - Nameplates (a dark rounded box, a name, a green or red bar) sit above the pillars. Nearer rows are larger in screen position but keep a constant size. Back rows' plates are stacked upward over the front rows' plates, and none overlap.
  - Past 22 m the plates show the name only.
  - Plates behind the wall are faded to about 30 %. Some back plates are absent because declutter hid them.
  - About 2,000 white damage numbers (gold and larger for crits) are popping, rising and fading over the pillars, with dark outlines. They are smaller farther away and faded where a pillar or the wall is in front.
  - The "Extraction / 20 m" tag is at the bottom centre edge, with a gold triangle arrow pointing down (the point is behind the camera).
  - The terminal is at the left: a green-on-black panel reading "TERMINAL 07 // GATE CONTROL", "Gate: SEALED", "Presses: 0" and a primary "Toggle gate" button, on a plane facing the camera in a dark bezel on a stand.
  - The HUD is at the top left, reading "HP 23 / 100" with a bar and the counts.
  - The loadout card is at the top right: "LOADOUT", a 300 px viewport showing a blue figure with a gold belt and visor on a plinth against violet, "Drag to turn", and the "Claim reward" button.
  - A red vignette pulses at the screen's edges, over the scene and under the HUD.
- `terminal-pressed.png`: the terminal reads "Gate: OPEN" in green and "Presses: 1".
- `confetti.png`: about 90 confetti rectangles (gold, pink, cyan, green, violet) fly up out of the Claim button and fall, spinning and fading. The button reads "Claimed ×1".
- `viewport-turned.png`: the figure in the viewport is turned about 70° further than at idle.

## Gfx scenes and perf test (for macrunner)

- og-tests gfx scenes `ui-world` (mass numbers, bars, text, icons, confetti, occluders) and `ui-panel` (a layer drawn into a render target, 2 frames) need `metal` references.
- Perf (§15.1, HUD plus 2,000 mass elements plus 100 anchored elements ≤ 1 ms CPU a tick):
  `cargo test -p og-ui --release --test tests world::perf -- --ignored --nocapture`
