<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-LightMask2D"></a>

### `og.LightMask2D` (component)

The light layers that light the entity's sprite, flipbook, 2D rig or tilemap: a Light2D lights it when their `layers` share a bit. Without one an entity is on layer 0 (bit 1). At most four different masks are lit in one frame; any further mask is lit as layer 0.

Token `og.LightMask2D`, registered under the schema name `og.LightMask2D`; its value type is `LightMask2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `layers` | `number` | `1` | — | — | — | A bit per light layer. |

```ogt
component LightMask2D {
  doc "The light layers that light the entity's sprite, flipbook, 2D rig or tilemap: a Light2D lights it when their `layers` share a bit. Without one an entity is on layer 0 (bit 1). At most four different masks are lit in one frame; any further mask is lit as layer 0."
  field layers u32 "A bit per light layer." {
    default 1
  }
}
```

<sub>`api.ogt` line 5703</sub>
