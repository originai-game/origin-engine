<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimelineKey"></a>

### `TimelineKey` (struct)

A keyframe: a value at a time.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |
| `value` | `number` | `0` | — | — | — | Its value. |
| `ease` | `KeyEase \| null` | `null` | — | — | — | How the segment into it moves; null is the out ease. |

```ogt
struct TimelineKey {
  doc "A keyframe: a value at a time."
  field atMs f32 "Its time." {
    unit ms
  }
  field value f32 "Its value."
  field ease Option.KeyEase "How the segment into it moves; null is the out ease."
}
```

<sub>Module `ui` line 2594</sub>
