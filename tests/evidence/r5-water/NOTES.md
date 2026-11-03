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
draws, never read from the GPU).

Without a renderer it runs to `tick 66 hash 5bc76283ccd85d37 entities 6`
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
