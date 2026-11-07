# Pixel VFX

Pixel-art 2D effects (explosions, slashes, spells, portals, beams, auras, sparks, heals, level-ups, weather, loot) as `type vfx2d` files: a Pixel VFX Forge recipe, layered heat fields cut into hard colour bands, that the cook renders into a flipbook sheet with an `impact` event.

The sheet it cooks into is an ordinary flipbook: `og api show anim`
(og.load.flipbook, og.Flipbook, og.FlipbookEvent) and `og api show
sprite` draw and play it. A 3D or mixed effect is a `type vfx` composed
effect (`og api show vfx`).

## 1. The file

An `.ogt` under `assets/` that starts `type vfx2d`, `version 1`, `---`,
then optional lines, then the recipe:

```text
type vfx2d
version 1
---
duration 0.8          # seconds the clip lasts (default: frames / fps)
loop false            # the clip's mode, loop or once (default: the recipe's loop)
impact 0.2            # = event 0.2 "impact", at most one
event 0.5 "linger"    # a FlipbookEvent with this name
recipe "slash.json"   # the recipe file, beside this one (or an asset id)
```

Instead of the `recipe` line, the recipe's JSON may follow inline, from a
line that starts with `{` to the end of the file. The recipe is the
Forge's JSON unchanged: a recipe exported from the Forge's web tool, or
written by hand, cooks as it is, and its layers keep their own `type`
field. Editing the recipe file cooks the effect again.

The cook writes the sheet `<id>` (one clip named after the file, the
recipe's fps, `filter nearest`, pivot the recipe's `pivot`) and its
texture `<id>#sheet`, the frames in one row when they fit in 4096 texels
(the Forge's own sheet layout). Rendering is deterministic: the same
recipe gives the same bytes on every host, the frames the Forge's
renderer gives.

## 2. The impact frame and events

Every pixel effect's clip sends an og.FlipbookEvent named `impact` on its impact frame: start og.time.hitStop, og.camera.shake, the target's flash and damage numbers from it.

The four optional lines mean what they mean in a `type vfx`. Times are
seconds from the start, put on the frame that starts at or after them;
a time past the duration is an error. Without an `impact` line, the
impact frame is the recipe's `impact` (0 to 1 of the effect) or else the
Forge's own pick: the frame with the largest jump in covered pixels in
the first 60% of the frames (the flash, or the burst's first expansion).
`event` lines add named events (at most 16 with the impact).

## 3. The recipe

A JSON object; unknown fields are ignored, so notes can stay in it.

| Field | Default | Meaning |
|---|---|---|
| `name` | `"Untitled effect"` | its display name |
| `w`, `h` | 64, 64 | canvas pixels (8 to 512); positions and sizes are in half-canvas units of the shorter side, y down |
| `frames`, `fps` | 16, 15 | frame count (1 to 120) and rate (1 to 60) |
| `loop` | false | seamless loop: periodic settings complete whole cycles |
| `seed` | 1 | every random choice |
| `ramps` | main fire, alt fire, smoke smoke, spark white | the ramps layers name as `$main`, `$alt`, `$smoke`, `$spark` |
| `hue`, `sat` | 0, 1 | rotate every ramp's hue (degrees), scale its saturation |
| `outline` | `"none"` | `"outside"`, `"outside8"` (with diagonals) or `"inside"`, in each layer's darkest colour |
| `outlineColor` | null | a fixed outline colour, `"#1a1020"` |
| `dither` | 0 | 0 to 1: a 4x4 ordered dither on band edges |
| `palette` | null | lock every pixel to the nearest (OKLab) colour of `"pico8"`, `"sweetie16"`, `"endesga32"`, `"gameboy"`, `"onebit"` or a list of hex colours |
| `cleanup` | true | remove isolated pixels and one-pixel holes from shape layers |
| `pivot` | [0.5, 0.5] | the canvas point (0 to 1 from the top left) that sits on the sprite's position |
| `impact` | auto | the impact as a 0 to 1 time |

A ramp is a name (`fire`, `lava`, `ice`, `frost`, `lightning`, `thunder`,
`poison`, `holy`, `dark`, `void`, `arcane`, `nature`, `water`, `blood`,
`earth`, `smoke`, `dust`, `ash`, `wind`, `plasma`, `energy`, `gold`,
`solar`, `shadow`, `steel`, `white`, `rainbow`), `"hue:<degrees>"` (a
hue-shifted ramp of seven) or a list of hex colours, dark to light.

Every layer takes `type`, `ramp` (default `$main`), `t0` and `t1` (when it
plays, 0 to 1 of the effect), `x` and `y` (its anchor), `angle`
(degrees), `squash` (vertical scale; 0.3 to 0.45 lays a ring on the
ground), `gain` (heat; lower uses the darker half of the ramp), `cut`
(0.1: heat below is clear), `gamma`, `dither`, `glow` (1 to 3 pixels of
stepped halo), `blend` (`"under"` paints only where nothing is drawn),
`hidden` and `seed`. Layers paint in order: smoke first, flashes last.

## 4. Layer types

The 22 types, defaults in brackets:

| Type | What it draws | Its fields |
|---|---|---|
| `burst` | an expanding fireball that cools and erodes into chunks | `r0` [0.08], `r1` [0.8], `grow` [0.5], `rough`, `freq`, `evolve`, `cool` [0.6], `erode` [1], `erodeStart` [0.3], `wobble`, `hollow`, `ease` [outCubic] |
| `ring` | a shockwave ring that breaks up | `r0`, `r1`, `th` [0.2], `thEnd`, `breakup` [0.7], `freq`, `fade`, `inner`, `ease` [outQuart] |
| `flame` | a rising flame, a body aura with `hollow` 0.8 to 1; loops | `w`, `hgt` [1.2], `taper`, `freq`, `rough`, `speed` [1], `flick`, `hollow` |
| `beam` | a horizontal beam from the anchor (turn it with `angle`) | `len` [1.8], `th`, `cap`, `wob`, `speed` [2], `grow` |
| `slash` | a crescent sword arc | `r` [0.72], `th`, `a0` [-160], `a1` [40], `sweep` [0.3], `trail`, `erode` |
| `orb` | an energy ball with a swirling core; loops | `r` [0.32], `pulse`, `pulseRate`, `spin`, `freq`, `rim` |
| `comet` | a projectile head and streaming tail; `from` [x, y] and `until` fly it to the anchor | `r` [0.2], `len` [1.1], `speed`, `rough`, `from`, `until` [0.45] |
| `vortex` | a spiral disc, portal or black hole (`core: "dark"`); loops | `r` [0.85], `rIn`, `arms` [3], `twist`, `spin`, `dir`, `core` |
| `pillar` | a column rising from a ground anchor | `w`, `hgt` [1.8], `speed`, `rough`, `grow` |
| `rays` | radial light rays | `count` [10], `r`, `width`, `jitter`, `inner`, `spin`, `twinkle` |
| `glyph` | a magic circle drawn on, then spinning | `r`, `rings` [[1, 0.84, 0.5]], `star` [5], `step` [2], `ticks` [24], `runes` [8], `draw`, `spin`, `fill` |
| `puffs` | smoke, dust, gas, or a breath cone | `count` [7], `spread`, `flat`, `rise`, `r0`, `r1`, `life`, `emit`, `dir` [-90], `cone`, `speed`, `drift`, `dissolve`, `decay`, `freq` |
| `particles` | sparks and debris: `mode` burst, spray, rise, fall, ambient, orbit or implode; `shape` dot, square, plus, cross, star, diamond, ring, bubble, streak, drop, shard, leaf, petal, chevron, coin, chunk or circle | `count` [16], `speed`, `life`, `emit`, `dir`, `spread` [360], `area`, `gravity`, `drag`, `ground`, `size`, `heat`, `twinkle`, `wiggle`, `orbitR`, `turns`, `implodeR`, `swirl`, `fadeIn`, `driftX`, `fallScale`, `spanX`, `spanY` |
| `bolt` | lightning, regenerated every `hold` frames: `mode` radial, strike (from the top edge) or line | `count` [4], `len`, `jag`, `depth` [5], `branch`, `thick`, `flicker`, `fadeStart`, `hold` [2] |
| `cracks` | ground cracks from the anchor | `count` [6], `len`, `squash` [0.4], `grow`, `fadeStart` |
| `spikes` | faceted crystal or rock spikes erupting, then shattering | `count` [5], `spread`, `hgt`, `width`, `lean`, `fan`, `grow`, `hold` [0.55] |
| `star` | an anime impact star that pops and hollows out | `points` [8], `r`, `inner`, `sharp`, `jitter`, `pop` [0.22], `hollow` [0.9] |
| `streaks` | `mode` claw (claw marks), speed (speed lines) or flow (wind arcs) | `count`, `angle`, `len`, `gap`, `curve`, `th`, `stagger`, `spanY`, `speed`, `r`, `squash`, `turns`, `dir` |
| `bubble` | a shield dome with hex cells and a travelling highlight; loops | `r` [0.8], `th`, `hex`, `cell`, `fill`, `shimmer` |
| `flash` | a white-hot disc for the first frames | `r` [0.6], `dur` [0.12] |
| `text` | 3x5 pixel-font text that pops and floats (damage numbers, CRIT!, LEVEL UP) | `text`, `scale` [1], `rise`, `pop` [true], `fadeStart` |
| `tornado` | a funnel with wrapped bands; loops | `w0`, `w1`, `hgt`, `twist`, `spin`, `sway`, `rough` |

## 5. Building an effect

- **One dominant shape.** A disc, a ring, a crescent, a column, a cone,
  read at 1x; sparks, smoke and debris never compete with it. Hide the
  secondary layers: it still reads, or it is not built yet.
- **Timing.** An optional 2 to 4 frame anticipation (`particles` implode,
  a contracting `ring`), a 1 or 2 frame `flash` at `t0` 0, a fast eased-out
  expansion (3 to 6 frames), then a slower break-up (`erode`, `breakup`,
  `dissolve`, `cool`). Impacts read at 8 to 10 frames and 18 fps, slashes 12
  at 18, explosions 16 to 20 at 15, loops 8 to 24 at 12.
- **Colour.** Ramps of 4 to 7 colours, hue-shifted; two colours an effect
  at most, plus smoke. Not every layer at `gain` 1: the core takes the hot
  end, secondary layers 0.5 to 0.8, or the effect is a white blob.
- **Size.** The effect's pixel density is the character's: a 64x64 effect
  beside a 96x96 character. Change `w` and `h`; never resample the frames.
- **Loops** meet their first frame when `speed`, `spin`, `turns` and
  `twinkle` are whole numbers and particles have `life` below 1.
- Check the first frames (anticipation, flash), the impact frame and the
  last frames: the effect breaks apart rather than shrinking.
