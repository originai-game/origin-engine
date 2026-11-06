<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-PassSize"></a>

### `PassSize` (struct)

The entity-id pass's size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | — | — | Width in pixels. |
| `height` | `number` | `0` | — | — | — | Height in pixels. |

```ogt
struct PassSize {
  doc "The entity-id pass's size."
  field width u32 "Width in pixels."
  field height u32 "Height in pixels."
}
```

<sub>`api.ogt` line 24552</sub>
