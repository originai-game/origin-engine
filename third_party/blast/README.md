# NVIDIA Blast (stress solver, support-graph model)

Source: https://github.com/NVIDIA-Omniverse/PhysX, `blast/`, revision
`b97365c6ddb32fda8469bc788ec18024862b0368`. Licence: BSD-3-Clause,
Copyright (c) 2008-2025 NVIDIA Corporation (`LICENSE.md` here, the
repository's licence, which covers `blast/`).

What Origin ports, and where:

| Origin | Blast |
|---|---|
| `crates/og-destruct/src/stress.rs` `StressProcessor` (prepare, solve, the coupling and bond matrices, CGNR) | `blast/source/shared/stress_solver/stress.cpp`, `bond.h`, `coupling.h`, `math/cgnr.h` |
| `stress.rs` `bond_stress`, `StressLimits::damage` | `blast/source/sdk/extensions/stress/NvBlastExtStressSolver.cpp` (`calcSolverBondStresses`, `generateFractureCommand`'s stress multiplier) |
| `crates/og-destruct/src/lib.rs` the asset model (support chunks, a chunk hierarchy, bonds as centroid, normal and area, bonds to the world), bond health as remaining effective area, radial damage falloff | `blast/docs/api/introduction.rst` (Support Model), `NvBlastExtDamageShaders` (`NvBlastExtRadialDamageDesc`) |

Changes: scalar f32 only (Blast's SISD path); the mass scale is the
arithmetic mean of the masses instead of the geometric mean (`ln` and `exp`
are not the same bits on every host); a scalar inertia per node; the world
is a static node per anchor, mirrored through the anchor face, so an anchor
bond has a length for its bending as a chunk-to-chunk bond has.

No Blast authoring code is used: the cook's Voronoi fracture is
`crates/og-geom/src/fracture.rs` (see `../three-pinata/`).
