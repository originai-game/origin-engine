# Break and restore, 3D SPH round (owed row 29)

Every new test was run against the code with one thing broken, seen to
fail, and passed again once the code was restored.

## og-physics (`crates/og-physics/tests/fluid3d.rs`, 6dcdc98)

| Broken | Failed (of 10) |
|---|---|
| no pressure | the zero-gravity drop, the viscous fluid spreading slower, the emitter and sink |
| no near pressure | the zero-gravity drop, the heavy fluid sinking, the poured block settling |
| no viscosity | the zero-gravity drop, the poured block settling, the viscous fluid spreading slower |
| an equal-mass split between a group's fluids | the zero-gravity drop, the heavy fluid sinking, the poured block, the emitter and sink, the terrain bowl |
| no sink | the emitter and sink |
| no height field collision | the terrain bowl |
| no wall friction | the terrain bowl |
| the 2D fluid's boundary strength (`TENSION` 1.0, 10 × this one's) | the heavy fluid sinking (a light layer trapped under the heavy) |

## og-render (`crates/og-tests/tests/fluids3d.rs`, 2f9e908)

| Broken | Failed |
|---|---|
| no absorption (transmission 1) | the drop in its particles' colours, the pool tints and shows through, the reference frame |
| no refraction | the pool refracts what lies behind it, the reference frame |
| no smoothing | smoothing melts the spheres, the pool test, the reference frame |
| no local lights in the composite | the local light lights the fluid, the pool test, the reference frame |
| no per-particle colours | the drop in its particles' colours |
| the composite ignores the scene's depth, alone | nothing: the depth pass's own discard behind the scene keeps the fluid off the wall |
| the depth pass's scene-depth discard, alone | nothing: the composite's depth test hides it |
| both of those | the basin wall hides the fluid behind it |

The wall test is kept although each half alone passes: it fails when the
fluid is drawn over a nearer surface, which needs both to be lost.

## fx-showcase (`templates/fx-showcase/tests/fluids3d.og.ts`, 332ec84)

| Broken | Failed |
|---|---|
| the honey at the pool's density (3 → 1) | "the heavy orange fluid sank under the blue" (orange 0.430 m, blue 0.376 m); the frame (4.399 % outliers, RMSE 0.03684) |
| the pool not drawn (`visible: false`) | "the pool's right end is blue" (163,116,106); the frame (14.474 %, RMSE 0.10335) |
| the honey without `bounds` | "no particle left the basin" (got 4); the frame (1.092 %, RMSE 0.01829) |

Restored, the station passes its 12 checks on metal and vulkan-cpu.
