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

## Batch 2: mesh, lit and distorting particles; sub-emitters and events

`particles/` is a low sun from the right over a striped wall and the
ground before it with four systems:

- `smoke` (left): `lit true` alpha puffs rising slowly;
- `debris` (centre): forty lit tetrahedra (`render mesh tetrahedron`)
  thrown up every second, tumbling, bouncing on the ground (an og.Collider
  box) and kicking two dust puffs at each impact (`subEmitter collision
  "fx/dust.ogt" 2 0.1`); its `events birth death collision` line makes the
  tick simulate it;
- `chips` (front right): small lit cubes (`render mesh box`), a fountain;
- `haze` (right): `distortion 0.02` puffs over a glowing brazier.

Gameplay reads og.ParticleEvent: a `Count` system tallies the debris'
births, deaths and impacts in the `Tally` resource and grows a brass
column (left of the crater) by 4 mm per impact.

Without a renderer it runs to `tick 121 hash 04b9e146816fa04c entities
26` (OG_RENDERER=none on the orb), the same on every run, with
`get:Tally` at tick 120 answering births 120, deaths 39, bounces 197.
The hash moves with the tally column, so it covers the events.

Commands (macrunner, a native dev build):

```sh
og run tests/evidence/r7/particles --headless --size 1280x720 --out /tmp/r7 \
  --commands "1.5:screenshot:particles;1.5:get:Tally;2:screenshot:particles-later;2.01:quit"
```

`particles` should show:

- the smoke column bright on its right (sun) side and grey-blue on its
  left, not the flat white of unlit smoke;
- the debris as small solid brown tetrahedra in flight and lying around
  the crater, faces lit differently by the sun (flat shading, no
  billboards, nothing transparent), with soft tan dust puffs where they
  land;
- the chips as small lit grey cubes arcing to the front right;
- the wall's stripes wavering above the brazier (the haze bends them; no
  colour is added, no dark or bright square around a puff, and the haze
  stops at the wall's silhouette: nothing in front of the brazier is
  bent);
- the brass column left of the crater, taller in `particles-later`.

og-tests frames (gfx references, `scripts/check.sh --only test`):

- `particles` and `particles-cpu` (unchanged scenes): must stay within
  tolerance of their references (the particle module now carries the
  lighting code; billboards unlit as before).
- `particles-lit` (new): `og_tests::fx::particles_lit_scene` after 90
  ticks: the same four looks as the sample with the debris on the CPU path
  (its collision sub-emitter) bouncing off a `ParticleCollider::Plane`;
  `stats.particle_systems` 3, `particle_collision_skipped` 0.
