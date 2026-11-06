<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tilemap`](../tilemap.md) · [index](../index.md)

<a id="type-TileObject"></a>

### `TileObject` (struct)

An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `string` | `""` | — | — | — | Its type or class. |
| `position` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its top-left corner, in world units relative to the map's top-left corner (y up, so below the corner is negative). |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its width and height (0 0 for a point). |
| `center` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its centre, relative to the map's corner. |
| `properties` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Its custom properties, as text. |

```ogt
struct TileObject {
  doc "An object of a map (a Tiled object, an LDtk entity, an `object` block): spawn points, doors, triggers."
  field name String "Its name."
  field kind String "Its type or class."
  field position Vector2 "Its top-left corner, in world units relative to the map's top-left corner (y up, so below the corner is negative)." {
    unit m
  }
  field size Vector2 "Its width and height (0 0 for a point)." {
    unit m
  }
  field center Vector2 "Its centre, relative to the map's corner." {
    unit m
  }
  field properties Map.String "Its custom properties, as text."
}
```

<sub>Module `tilemap` line 74</sub>
