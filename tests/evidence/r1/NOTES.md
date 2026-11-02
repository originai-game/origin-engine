# R1: meshlets

## Batch 1: clusters in the cook

The cook cuts every level of every model primitive into clusters (64
triangles, 192 vertices, sphere, normal cone, error fields for a later
DAG). No frame moves; CPU tests only (`cargo test -p og-cook --test tests
lod::`, `cargo test -p og-render --lib meshlet`).

## Batch 2: GPU cluster culling and indirect draws

Under GPU culling the camera pass's opaque batches of clustered meshes
pick their level per instance on the GPU, cull their clusters (frustum,
normal cone, depth pyramid) and draw them by vertex pulling
(`docs/design.md` §2.6 Meshlets).

gfx tests (`crates/og-tests-cooked/tests/gfx/meshlets.rs`, new frames):

- `meshlet-forest`: 132 cooked trees on a grid receding from the camera,
  `debugView lod`: the near rows green (level 0), then yellow and orange
  farther back, every tree whole (no missing clusters, no holes, no
  stray triangles).
- `meshlet-occlusion`: a wall over the left two thirds of the view in
  front of the forest (3 frames, so last frame's occluders exist): the
  wall, and the right third's trees whole beside it.
- `meshlet-near`: one cooked model close up through the meshlet path; it
  is compared with the same scene through the regular path at the
  default tolerance, so it should look the same.

Commands (a native dev build), a cooked-model scene through the path,
plain and with the level tint:

```sh
og run templates/fx-showcase --headless --size 1280x720 --out /tmp/r1 \
  --commands "3:screenshot:plain;4:debug:lod;6:screenshot:lod;7:quit"
```

`plain` should match fx-showcase's reference within tolerance; `lod`
tints every model by its level.

## Batch 3: shadows through the meshlet path

Each cascade and each local light's shadow face is a view of the same
meshlet dispatch: clustered casters are culled per cluster against the
pass's planes (no cone, no pyramid) at the camera's level and drawn by
vertex pulling in the shadow pipeline (`meshlet_shadow.wgsl`).

gfx tests (new frame `meshlet-shadows`):

- `meshlet-shadows`: five trees on a plain from above and in front, a
  low sun from the left throwing long shadows to the right, and a warm
  shadowed spot over the middle tree throwing its shadow forward (towards
  the camera). Every shadow whole, the crowns' shadows dappled as the
  regular path's, no acne on the trees, no shadow missing.
  `meshlet_shadows_match_the_regular_path` compares it with the regular
  path at the default tolerance.
- `meshlet-near`'s comparison now covers its sun's cascades through both
  paths as well.

The same `og run` script as batch 2 exercises it (fx-showcase's sun).

## Batch 3b: the meshlet pass's own two phases

The camera's clusters are culled in two phases: those visible last frame
are drawn first into GPU culling's occluder depth, the pyramid is built
again, and the rest are tested against it (`docs/design.md` §2.6).

- `meshlet-occlusion` (3 frames): unchanged in what it shows (the wall on
  the left two thirds, the right third's forest whole). New:
  `meshlet_occlusion_matches_the_regular_path` compares it with the
  regular path at the default tolerance, so a cluster neither phase draws
  fails as a hole.
