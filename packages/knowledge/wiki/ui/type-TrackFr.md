<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TrackFr"></a>

### `TrackFr` (struct)

A share of the space the other tracks leave.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fr` | `number` | `1` | — | — | — | Its share. |

```ogt
struct TrackFr {
  doc "A share of the space the other tracks leave."
  field fr f32 "Its share." {
    default 1
  }
}
```

<sub>Module `ui` line 1610</sub>
