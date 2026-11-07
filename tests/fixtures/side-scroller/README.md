# side-scroller

A night cave in the manner of Hollow Knight and Dead Cells: a hooded knight
runs at a slime and slashes; the slash's hit frame blows the rock mound in
front of it apart. It exercises the 2D stack together on the real engine:

- **Flipbooks from GIFs.** `assets/hero/hero.ogt` cooks three generated GIFs
  (512×512, flat magenta ground) into one sheet: `key auto 0.1 holes` keys
  the ground and the pockets of it the slash's arc encloses, `align feet`
  holds the figure still, `scale 0.25` brings it down, and the idle (drawn
  larger) takes `scale 0.89` in its source block. `slime.ogt` is the enemy.
- **A sprite animation graph.** `KNIGHT` in `src/main.ts` (og.anim.graph
  with `sprite: true`) names the sheet's clips `idle`, `run` and `attack`;
  the `Drive` system sets its parameters and moves the knight in its run
  state. The attack clip's `hit` event (frame 3) is the blow.
- **Pixel terrain.** The floor, the roof and the mound are one
  og.PixelTerrain built with fill2d (rock over bedrock); the hit calls
  og.destruction.explode2d from the system, throwing og.PixelPiece debris.
- **2D light.** og.Lighting2D with two torches and a moon spot (og.Light2D,
  `castShadows`, `sourceRadius`), glowing flames and mushrooms.
- **Camera and parallax.** og.Follow2D (clamped) follows the knight; the
  cave's two far walls are one tilemap (`assets/bg/cave.ogt`) of two layers
  at parallax 0.3 and 0.6, the far one unlit.
- **Loose PNGs packed.** The props (`assets/props/*.png`) are the still
  sources of one sheet, `props.ogt`, with `crop each`.

`art.py` generates the walls, their tileset and map, the props, their sheet
and `src/art.ts` (each prop's cell size); rerun it to change them. The GIFs
are copied from the generated character set.

**vfx2d.** `vfx2d()` in `src/main.ts` is where the pixel effects go once
`type vfx2d` lands: `slash` on the hit frame at the blade's tip, `dust` on
each run step. Today it draws nothing; adding an effect is the commented
`og.vfx.spawn` line and an `assets/fx/<name>.ogt`.

What it checks (`og test tests/fixtures/side-scroller`, `tests/slash.og.ts`):
the graph reaches the attack state (`the knight slashes`) and returns to
idle (`state==0`), the run fires step events, the hit fires once and the
blast cuts loose at least one piece (`the mound breaks into pieces`,
`detached>=1`; 22 today); the frame at 2.0 s, the debris in the air beside
the torch, matches `references/metal/slash.png` (rmse ≤ 3).

Budget: under 10 s warm. Measured 1.8 s of run, 4.5 s with its cook and
check.

## Measured

`og sprite inspect` of the sources (key auto at 0.1; `--holes` as the sheet
keys them). Edge fringe is the keyed edge's texels, those of them partial,
and their mean step to the figure's interior; "near the key" counts texels
still drawn within the tolerance of the magenta (a fringe or a pocket):

| GIF | border (share) | drift centroid / bounds (px) | fringe edge, partial, step | near the key, without / with `holes` |
|---|---|---|---|---|
| hero_idle | #f604f4 (99.9 %) | 1.0 30.4 / l19 t39 r11 b0 | 15842, 14803, 0.303 | 385 / 0 |
| hero_run | #f803f6 (100 %) | 11.7 31.0 / l15 t16 r0 b29 | 15628, 13968, 0.286 | 9 / 0 |
| hero_attack | #f904f7 (99.7 %) | 55.0 19.9 / l20 t39 r105 b1 | 15005, 14239, 0.255 | 7806 / 0 |
| slime | #f803f6 (100 %) | 7.7 130.8 / l21 t131 r28 b94 | 7877, 7584, 0.299 | 0 / 0 |

Before the engine fix that came with this fixture, `key auto` refused
hero_idle (its magenta covered 28.8 % of the border as one 5-bit colour).
The attack's 7806 texels are the magenta inside the slash's arc, which the
border's flood never reaches; `holes` keys them.
