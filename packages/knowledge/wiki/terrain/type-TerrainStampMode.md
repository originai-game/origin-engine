<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainStampMode"></a>

### `TerrainStampMode` (enum)

What og.terrain.stamp does to the height field under a stamp's mesh.

A string: one of the variants below. The zero value is `"carve"`.

| Variant | Doc |
|---|---|
| `"carve"` | Lower the ground under the mesh to `depth` below its underside (the lowest of its triangles over each height sample), so the mesh is the surface there: a cliff's face and lip, an arch, an overhang. On the edge of what the mesh covers the ground goes down to the underside itself, so the ground beside the mesh meets its foot with no trench. Ground already lower is left as it is. |
| `"hole"` | Cut out every cell whose centre the mesh covers (seen from above), neither drawn nor collided with: the mesh stands in the ground's place (a cave mouth, a tunnel's portal). |

```ogt
enum TerrainStampMode {
  doc "What og.terrain.stamp does to the height field under a stamp's mesh."
  variant carve "Lower the ground under the mesh to `depth` below its underside (the lowest of its triangles over each height sample), so the mesh is the surface there: a cliff's face and lip, an arch, an overhang. On the edge of what the mesh covers the ground goes down to the underside itself, so the ground beside the mesh meets its foot with no trench. Ground already lower is left as it is."
  variant hole "Cut out every cell whose centre the mesh covers (seen from above), neither drawn nor collided with: the mesh stands in the ground's place (a cave mouth, a tunnel's portal)."
}
```

<sub>Module `terrain` line 26</sub>
