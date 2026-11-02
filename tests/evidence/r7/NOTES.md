# R7: decals and particles

## Batch 1: decals with normal maps, no 32 cap

`decals/` is a wall and the ground before it with 600 og.Decal bullet
holes (a frame drew 32 before R7), each with the atlas's normal map
(`normalMin` 0.5 0 to `normalMax` 1 1 of `textures/bullet-holes.png`, an
sRGB colour texture whose right half is the normal map), and two large
craters on the ground under a low sun from the left: `crater-a` with
`normalScale` 1, `crater-b` with 2.

Without a renderer it runs to `tick 12 hash c2a3be922ea877ac entities 605`
(OG_RENDERER=none on the orb; decals are presentation only, so the hash
does not depend on them).

Commands (macrunner, a native dev build):

```sh
og run tests/evidence/r7/decals --headless --size 1280x720 --out /tmp/r7 \
  --commands "0.1:screenshot:holes;0.2:quit"
```

`holes` should show:

- every one of the 600 small holes drawn, on the wall's face (12 rows of
  32) and on the ground before it (8 rows): a dark pit inside a pale
  chipped rim, none missing at the right or the top (the old cap kept the
  first 32, the bottom-left of the wall);
- the two craters at the front: rims lit on their left (the sun's) side and
  darker on the right, pits shaded the other way; the right crater's
  relief clearly stronger;
- no seams or blocky tiles where decals cross cluster boundaries (the
  screen's 16 × 9 tiles), no decal leaking off the wall's top edge.

Fix (macrunner's first draw of 03331ea6a): thin blue lines ran from the
ground's holes along their boxes' edges. A tap of the colour region's
filter past u 0.5 reached the atlas's normal-map half (opaque blue) at
grazing angles. Each decal sample is now clamped inside its region by its
filter's reach (`decal_sample` in pbr.wgsl). The bullet-hole normal map is
gentler (the pit's lip leans 40° at most, against about 60° before), so
`crater-b`'s normalScale 2 reads clearly deeper than `crater-a`.

og-tests frames (gfx references, `scripts/check.sh --only test`):

- `decals` (unchanged scene): must stay within tolerance of its current
  reference: the decal shading moved into storage buffers but computes the
  same values.
- `decals-many` (new): `og_tests::fx::decals_many_scene`, 640 holes in a
  32 × 20 grid on the wall and ground plus the two craters, sun from the
  left; `stats.decals` 642, `decals_dropped` 0. Same look as `holes`.
- `effects::sprites::decals_project_onto_surfaces_past_the_old_cap`: 45
  decals where the base scene's scorch is the 41st; asserts it still
  darkens the ground (it was dropped before), and that 260 decals on one
  spot draw with at least 5 cluster references dropped.
