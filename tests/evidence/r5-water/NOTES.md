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

Without a renderer it runs to `tick 66 hash 3edbbae47cf441f0 entities 6`
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
so they ride, pitch and roll. Each starts set down on the water at tick
0 (`og.physics.waterHeight`).

Without a renderer it runs to `tick 396 hash b346de0709ba562a entities
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
still on the surface; the boats have barely drifted sideways.
