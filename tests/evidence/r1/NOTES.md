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
