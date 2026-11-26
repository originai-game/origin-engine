# Geometric effects

Install with `og vfx add geometric`; spawn `vfx/geometric/<name>.ogt`.
Nine texture-free visual mechanisms for `minimal-geometric` and
`faceted-low-poly`. Hard mesh silhouettes, one coral accent and
slate neutrals; no bloom, noisy textures or soft billboard smoke.

The `layered-silhouette` variant is not supported: its full-frame depth-band
replacement hides these transparent effects. It remains a coverage gap.

| Effect | Placement |
|---|---|
| explosion | Ground origin; expanding broken facets and a roughly 3 m polygon shock front |
| impact | Surface origin, +y normal; angular chips and a short open pentagon |
| muzzle | Barrel origin, +y forward; tapered geometric pulse |
| pickup | Object origin; rising small cubes |
| magic | Source origin; rotating open polygon outlines |
| dust | Ground origin; low slate tetrahedra |
| splash | Water plane; ballistic pale facets |
| trail | Loop at a moving point; local axis +y, particles stay in world space |
| celebration | Project a UI anchor into the scene; a fan of coral chips |

This is a downloadable pack, not a gameplay template. No scoring, damage,
sound or UI logic. Explosion and impact include short camera impulses,
decayed by the game's `og.Camera.traumaDecay` setting. Celebration is world-space, not a UI
widget. `trail` leaves geometry where its moving instance has been; a
stationary preview shows its emission, not a controller moving it.

All compositions are original MIT work, Origin Engine. Mesh particle
mechanisms follow William T. Reeves, “Particle Systems—A Technique for
Modeling a Class of Fuzzy Objects”, ACM TOG 1983,
https://doi.org/10.1145/357318.357320 (method reference only; no paper text,
figures or third-party code copied). Built-in ring and tetrahedron meshes
avoid external assets entirely. Full provenance is in `assets.json`.
