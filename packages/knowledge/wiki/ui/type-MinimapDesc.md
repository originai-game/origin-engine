<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MinimapDesc"></a>

### `MinimapDesc` (desc struct)

A mini-map frame.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The map; null is the stage colour. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region of the map shown, [u0, v0, u1, v1]; null is the whole texture. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 160. |
| `markers` | `readonly MapMarker[]` | `[]` | — | — | — | Markers over the map. |
| `label` | `string` | `""` | — | — | — | A zone name under it; empty is none. |

```ogt
struct MinimapDesc {
  doc "A mini-map frame."
  desc true
  field texture Option.Texture "The map; null is the stage colour."
  field uv Option.Array.f32.4 "The region of the map shown, [u0, v0, u1, v1]; null is the whole texture."
  field size f32 "Its side; 0 is 160." {
    unit cssPx
    min 0
  }
  field markers List.MapMarker "Markers over the map."
  field label String "A zone name under it; empty is none."
}
```

<sub>Module `ui` line 1023</sub>
