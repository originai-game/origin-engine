# The realistic set

The cinematic 3D family's second asset set: the same beats as the
family's primitives (a hero, a tree, a crate, a lantern, the ground and a
far layer) as scanned and generated models, for the styles whose contract
is realism. `styles/cinematic-realism/set` and
`styles/muted-cinematic-3d/set` name it; `tools/art-samples/capture.ts`
lays it over the project (its `assets/models/` in place of the family's,
`beats.json`, `src/models.ts` and `src/main.ts`). Both styles take the
same unmodified files: the cook's conformance to each look (ArtTexels
density 512 for one, ArtShape characterFaces 10000 for the other) and the
look's frame make the difference. The family's base set stays the
primitives, so painterly-3d and layered-silhouette do not move.

| Model | Source | Packed |
|---|---|---|
| `hero.glb` | Meshy, through the origin MCP's `generate_3d` (meshy-6 `text_preview` then `text_refine` with PBR, from the brief's prompt), 415k triangles | 1.8 m, colour 1024, other maps 512, simplified to 21k triangles |
| `pine.glb` | Meshy, the same way, 2.7M triangles | 3.6 m, colour 2048 (density), other maps 512, simplified to 22k |
| `crate.glb` | Poly Haven `wooden_crate_01` (CC0), the origin asset catalogue (`grp_4824f0a4cc69401f82`, 1k maps); its 2k diffuse from Poly Haven itself (the catalogue carries 1k only; md5 checked against api.polyhaven.com) | 0.45 m tall, colour 2048, other maps 512 |
| `street-lamp.glb` | Poly Haven `street_lamp_01` (CC0), catalogue `grp_72afdda491b8739f3d` | 2.9 m, maps 512 |
| `hand-lantern.glb` | Poly Haven `Lantern_01` (CC0), catalogue `grp_17d5a11495f4dfec62` | 0.42 m, maps 256 |
| `stump.glb` | Poly Haven `tree_stump_01` (CC0), catalogue `grp_b97febd8d60e25c469` | 0.7 m, colour 1024, other maps 256 |
| `ground.glb` | Poly Haven `forest_ground_04` (CC0), catalogue `grp_4cde219f95811c3035`, diffuse and normal | `make3d.py ground`: 180 m, colour 1024 every 2 m (512 texels a metre), normal 512 |
| `ridge.glb` | `make3d.py ridge` | untextured, hazed by the look |

The catalogue's files come through `assets_list_files` and
`assets_download` (signed URLs valid five minutes); the Meshy ones through
`generate_3d`'s `poll`. Each model was packed with
`tools/art-samples/pack.py <source> <out.glb> <metres> [--color-tex N]
[--tex N] [--quality Q] [--image uri=file]`, then welded and simplified
with gltf-transform 4 (`weld`, `simplify --ratio R --error E`; no
extension the cook would need to read).
