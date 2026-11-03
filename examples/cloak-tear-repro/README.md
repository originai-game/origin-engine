# og.Garment tear repro (the traveller's cloak)

A static traveller (bind pose, no clip playing) with its cloak as an
og.Garment in a gusting og.Wind. Dark shards stick out of the cloak and
the seam gaps open from the first simulated tick; with `maxDistance: 0`
the cloak draws clean, so the tear is in the simulation, not the asset's
drawing.

    og run examples/cloak-tear-repro --headless --size 360x480 \
      --commands "0:orbit:2;0.02:screenshot:t01.png;1:screenshot:t60.png;1.05:quit"

`orbit:<0..3>` turns the camera (0 front, 1 side, 2 back, 3 other side).

Garment desc: `{ mesh: 1, maxDistance: 0.25, backstop: 0, bending: 0.3, iterations: 6 }`.
Wind: strength 0.5, direction (0.3, 0, 1), speed 6, gust 0.8.

## The asset (assets/models/traveller.glb)

The Meshy traveller (one skinned mesh, 24 joints, 0.01-scale armature)
split by `tools/split-cloak.ts`: glTF mesh 1 "Cloak", on the same skin,
COLOR_0 white with the freedom in alpha (0 at the seam to the body and
above 70 % of the height, rising to 1 by the hips). Simplified with
meshoptimizer `simplifyWithAttributes` to 25 % (3822 triangles, 5923
vertices, 2541 points welded by exact position), the seam locked.

Measured on it:
- Meshy models the cloak as a thick shell: an outer face and a lining
  about 1.25 cm behind it, facing the other way (4280 of 5923 vertices
  have an opposite-facing vertex within 3 cm), joined at a rim.
- Edge valence over welded points: 1408 boundary, 5008 interior, 2 with
  3 triangles and 9 with 4 (non-manifold, from the simplification).
- Two connected pieces: the cloak (3685 triangles) and a 137-triangle
  piece near the ground held nearly fixed (freedom ≤ 0.1).

Open by design: the cloak's front opening and its hem (the shell's rim
there), and the seam where it meets the body (pinned, freedom 0). The
non-manifold edges and any other open seam are artefacts of the split.

Tried without a fix: backstop 1.0 (shards remain); a weld normal turned
away from the skinning joints (worse); dropping the lining by geometry
(shredded the sheet).
