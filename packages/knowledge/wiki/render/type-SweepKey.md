<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SweepKey"></a>

### `SweepKey` (struct)

A value at a fraction of a sweep's path: linear between keys, held before the first and past the last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `t` | `number` | `0` | — | `ratio` | — | The fraction of the path's length, 0 to 1. |
| `value` | `number` | `0` | — | — | — | The value there. |

```ogt
struct SweepKey {
  doc "A value at a fraction of a sweep's path: linear between keys, held before the first and past the last."
  field t f32 "The fraction of the path's length, 0 to 1." {
    unit ratio
  }
  field value f32 "The value there."
}
```

<sub>Module `render` line 3556</sub>
