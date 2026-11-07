<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SweepGeometryDesc"></a>

### `SweepGeometryDesc` (desc struct)

A section swept along a path, scaled and twisted over it: horns, tusks, handles, blades, tapered pipes, twisted columns (img2threejs's tapered sweep; three.js has no class for it). The section's x lies along the path's normal and y along its binormal (three.js's Frenet frames), rings evenly spaced along the path's length; u runs along the path, v round the section (flipped to run down the image), the sides smooth. An open path's ends are capped.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `section` | `Path2 \| readonly Vector2[] \| null` | `null` | — | — | — | The section's outline: an og.Shape or og.Path2 (flattened by curveSegments; holes are not swept) or its points, at least three, either winding. |
| `path` | `Path` | `0` | — | — | — | The path (og.path.create or og.path.find). |
| `steps` | `number` | `64` | ≥ 1 | — | — | Rings along the path, less one. |
| `scale` | `readonly SweepKey[]` | `[]` | — | — | — | The section's scale over the path (none: 1 throughout). A horn: [{ t: 0, value: 1 }, { t: 1, value: 0.05 }]. |
| `twist` | `readonly SweepKey[]` | `[]` | — | — | — | The section's turn round the path over its length, in radians (none: 0). |
| `caps` | `boolean` | `true` | — | — | — | Close the ends of an open path. |
| `curveSegments` | `number` | `12` | ≥ 1 | — | — | Points per curve when an og.Shape section is flattened. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct SweepGeometryDesc {
  doc "A section swept along a path, scaled and twisted over it: horns, tusks, handles, blades, tapered pipes, twisted columns (img2threejs's tapered sweep; three.js has no class for it). The section's x lies along the path's normal and y along its binormal (three.js's Frenet frames), rings evenly spaced along the path's length; u runs along the path, v round the section (flipped to run down the image), the sides smooth. An open path's ends are capped."
  desc true
  field section Option.OneOf.Path2.List.Vector2 "The section's outline: an og.Shape or og.Path2 (flattened by curveSegments; holes are not swept) or its points, at least three, either winding."
  field path Path "The path (og.path.create or og.path.find)."
  field steps u32 "Rings along the path, less one." {
    default 64
    min 1
  }
  field scale List.SweepKey "The section's scale over the path (none: 1 throughout). A horn: [{ t: 0, value: 1 }, { t: 1, value: 0.05 }]."
  field twist List.SweepKey "The section's turn round the path over its length, in radians (none: 0)."
  field caps bool "Close the ends of an open path." {
    default true
  }
  field curveSegments u32 "Points per curve when an og.Shape section is flattened." {
    default 12
    min 1
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>Module `render` line 3311</sub>
