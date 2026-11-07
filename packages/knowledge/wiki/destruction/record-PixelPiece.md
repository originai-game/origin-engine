<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="record-PixelPiece"></a>

### `og.PixelPiece` (component)

A piece of an og.PixelTerrain cut loose (an island carve2d cut free, a piece explode2d threw or fracture2d broke off), written by og.destruction2d: a dynamic og.RigidBody whose og.Collider is the convex hull of its cells (SHAPE_CONVEX, a polygon in 2D), drawn as its cells at its og.Transform (the bottom-left corner of its cell (0, 0), turning with the body). Its cells are world state; despawn it to clear it away.

Token `og.PixelPiece`, registered under the schema name `og.PixelPiece`; its value type is `PixelPiece`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `terrain` | `Entity` | `0` | — | — | read only | The og.PixelTerrain it was part of. |
| `width` | `number` | `0` | — | — | read only | Its cells across. |
| `height` | `number` | `0` | — | — | read only | Its cells up. |
| `cells` | `number` | `0` | — | — | read only | Its solid cells. |
| `tick` | `number` | `0` | — | — | read only | The tick it broke loose. |

```ogt
component PixelPiece {
  doc "A piece of an og.PixelTerrain cut loose (an island carve2d cut free, a piece explode2d threw or fracture2d broke off), written by og.destruction2d: a dynamic og.RigidBody whose og.Collider is the convex hull of its cells (SHAPE_CONVEX, a polygon in 2D), drawn as its cells at its og.Transform (the bottom-left corner of its cell (0, 0), turning with the body). Its cells are world state; despawn it to clear it away."
  field terrain Entity "The og.PixelTerrain it was part of." {
    readonly true
  }
  field width u32 "Its cells across." {
    readonly true
  }
  field height u32 "Its cells up." {
    readonly true
  }
  field cells u32 "Its solid cells." {
    readonly true
  }
  field tick u32 "The tick it broke loose." {
    readonly true
  }
}
```

<sub>Module `destruction` line 380</sub>
