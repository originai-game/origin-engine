# R5: water (the sea and flow maps)

## Batch 1: the sea (og.Water.wind) and flow maps (og.Water.flowMap)

`sea/` is an og.Water 6 km across (its grid finest at the centre, under
the camera) raised by a 9 m/s wind running away
from the camera and to the left (`wind -3 -8.5`, choppiness 0.6: waves
about 65 m long at the peak, a significant height of 1.8 m), under the
physical sky (og.PhysicalSky) at 15 h with a light haze, seen from 4 m
up. Five red buoys stand across the view 15 to 55 m out; the `float`
command sets each one's y to `og.physics.waterHeight` at its point, the
sea the tick computes with og_core::ocean (the same waves the renderer
draws, each faded where the grid's cell under the buoy is too coarse for
it, as the renderer's vertices fade it: a buoy rides the drawn surface,
never read from the GPU).

Without a renderer it runs to `tick 66 hash 50a99c57712590c6 entities 6`
with `--commands "0.5:float;1.0:float;1.1:quit"` (OG_RENDERER=none on the
orb).

Commands (macrunner, a native dev build):

```sh
og run tests/evidence/r5-water/sea --headless --size 1280x720 --out /tmp/r5 \
  --commands "1.0:float;1.0:screenshot:sea.png;2.5:float;2.5:screenshot:sea-later.png;2.6:quit"
```

`sea` should show:

- open water to a hazy horizon a little above the frame's middle, no rim
  of the surface or band of sky below the horizon;
- long swells running away from the camera and to the left, their faces
  lit and shaded (darker troughs, brighter crests catching the sky), the
  crests a little sharper than the troughs, fine ripples on top; no grid
  pattern, no flat facets, no shimmering stair-steps toward the horizon;
- the five red buoys sitting on the surface, each half submerged at the
  height of the water around it (not floating above the swell nor sunk
  under it), at different heights as the swell passes them.

`sea-later` (a second and a half on): the swells have moved on (each crest
a few metres further along the wind), the buoys at new heights, still on
the surface.

## Buoyancy (og.Buoyancy)

`boats/` is the storm frame's gale (og.Water.wind 7 −18.7, 20 m/s,
choppiness 1: waves some 330 m long, a significant height near 10 m)
under a dark overcast (og.Clouds coverage 0.97, grey) at 13 h, seen from
10 m up. Three boats (box hulls 2.4 × 1 × 6 m of density 0.6, with a
drawn cabin aft of the middle; one red, two white) and four orange
buoys (spheres of radius 0.6, density 0.8) carry og.Buoyancy and nothing
else moves them: the og.physics step lifts each hull's cells under the
sea it computes in the tick (og_core::ocean, the waves the grid draws),
so they ride, pitch and roll. The lift is the displaced water's weight
plus the water's own acceleration (Froude–Krylov) and the drag acts
against the water particles' velocity, so a floater moves with the
water it sits in: heave up to about 5 m/s on these crests is the sea's
own (its particles' orbital speed), and the hull stays wet through
them. Each starts set down on the water at tick 0
(`og.physics.waterHeight`).

Dumped every 0.5 s to 6.5 s (`og.Buoyancy.submerged`): the boats keep
0.11 to 0.24 at least, mean 0.30 (their rest value); the buoys 0.25 to
0.39 at least, mean 0.40.

Without a renderer it runs to `tick 396 hash 33b74a49443d3643 entities
11` with the commands below (OG_RENDERER=none on the orb, a fresh
og-host-node in OG_NATIVE_ADDON).

```sh
og run tests/evidence/r5-water/boats --headless --size 1280x720 --out /tmp/r5 \
  --commands "5.0:screenshot:boats.png;6.5:screenshot:boats-later.png;6.6:quit"
```

`boats` should show:

- the gale as the storm frame draws it (grey overcast, rough grey-green
  sea, whitecaps), the horizon a little above the frame's middle;
- three box boats 30 to 70 m out at different headings, each sitting in
  the water about a third of its hull deep (the waterline cutting its
  sides, never floating above the surface nor buried under it), tilted
  with the slope of the swell under it: bow up or down, rolled to one
  side, the three at different heights and angles;
- four orange balls a little under half submerged, at the height of the
  water around each.

`boats-later` (1.5 s on): every boat and buoy at a new height and tilt,
still in the water (none lifted clear on a crest); the boats have barely
drifted sideways.

## The FFT sea: re-pinned hashes

og.Water.endless (the FFT sea's batch 1 and 2) changed og.Water's layout,
which the world hash covers, so both samples' final hashes moved: the sea
from 3edbbae47cf441f0 to 50a99c57712590c6, the boats from 703f695dab0d1ae5
to 33b74a49443d3643. Nothing the tick computes moved: `dump_world` of the
boats at 3.0 s and 6.5 s, with the addon before and after the field, differ
in the systems' timings and the hash alone (every entity's components,
og.Buoyancy's submerged share included, are the same).

## Waterline: the floaters against the drawn sea

`boats/waterline.txt` holds the floaters' poses at 5.0 s and 6.5 s (ticks
300 and 390), from the run's `dump_entity`:

```sh
og run tests/evidence/r5-water/boats --headless --out /tmp/wl --commands \
  "5.0:dump_entity:boat-1:boat-1-5.0.json;...;6.5:dump_entity:buoy-3:buoy-3-6.5.json;6.6:quit"
```

og-tests' `the_floaters_sit_in_the_drawn_sea` (ocean.rs, a GPU test) draws
the boats' sea at those ticks and reads the drawn surface's height back
from the GPU under each buoy's centre and each hull's four keel corners
(`Renderer::sea_heights`: the clipmap's vertices' displacement, the rest
point found by undoing the crests' lean). It asserts that the tick's waves
are drawn as og.physics.waterHeight floats the bodies on (to 2 cm) and that
every buoy crosses the drawn surface, and prints every clearance:

```sh
cargo test -p og-tests --test tests the_floaters_sit -- --nocapture
```

Before the foam batch the FFT's middle cascade began at 0.21 rad/m, inside
the band the tick floats bodies on (to 0.39 rad/m at its 4 m cell there),
so the sea's 16 to 29 m waves were drawn twice: on the CPU's estimate a
keel corner stood 10 cm over the drawn trough. The cascades are now parted
where the tick's waves end at its finest cell (π/2 over the cell). Against
the tick's surface every keel corner is 9 to 44 cm under and every buoy's
centre 8 cm over; the cascades' 3 to 14 m chop (render only) still moves
the drawn surface ±0.3 m under them, which bares one corner (boat-0 at
6.5 s) by about 6 cm on the CPU's estimate.

Hull wave suppression closes that: og.Buoyancy's step records each hull's
footprint (the rectangle its own axes span over the water) in the sea's
`Wakes` (`record_sea`, the same path og.WaterSurface's ripples read; not
world state), og.Water carries them to the renderer (`Water::wakes`), and
water.wgsl `hull_calm` leaves the FFT's chop out under each hull, fading
back in 1.5 m beyond it. Under a hull the drawn surface is then the
tick's, and the test asserts no keel corner stands 5 cm clear of it.

## Foam (the FFT sea's batches 3 and 4)

Ported from web-ocean (third_party/ocean/): two world foam levels about the
camera (64 m and 512 m, 256 texels) in the fields array's layers after the
cascades, stepped every 1/30 s of the tick's time. Each step injects foam
where the drawn surface folds, decays it (τ 4.5 s) and drifts it downwind.
A crest is white where its Jacobian falls under the level that a share of
the surface falls under: Monahan's whitecap coverage at the wind
(`foam_thresholds`; 0.7 % at the breeze, 10 % at the gale).

Frames never depend on the wall clock. On a fresh renderer, a new sea, a
jump back in time (a restore) or a jump forward of more than 2 s, the foam
starts empty and is warmed up over 5τ (22.7 s) of tick time before the
frame: every 8th step, then every step of the last 2 s, about 140 steps,
each an FFT at its own time plus the foam update.
