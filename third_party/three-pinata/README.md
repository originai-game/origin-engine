# three-pinata (Voronoi fracture method)

Source: https://github.com/dgreenheck/three-pinata, revision
`26c746f993658f903fa0c715abdf87c1fedc2379` (the three.js port of the same
author's OpenFracture for Unity). Licence: MIT, Copyright (c) 2023 Daniel
Greenheck (`LICENSE` here).

`crates/og-geom/src/fracture.rs` follows its method, not its code: a cell
per seed as the half-space intersection of the bisecting planes between its
seed and every other seed (`lib/src/fracture/VoronoiCell.ts`), cut faces
with their own material and generated uvs, a cell that a non-convex mesh
leaves in separate parts split into chunks (`FractureFragment.ts`
`findIsolatedGeometry`), and seeds gathered round an impact point
(`utils/SeedPointGenerator.ts` `generateImpactBased`). Where three-pinata
slices the mesh one plane at a time and triangulates each cap, Origin builds
the convex cell first and intersects the mesh with it through og-geom's
exact CSG (Shewchuk's predicates), so every chunk is closed and the cook's
output is the same bits on every host.
