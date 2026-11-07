<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BannerResult"></a>

### `BannerResult` (struct)

Where a banner is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | On screen. |
| `done` | `boolean` | `false` | — | — | — | It finished this tick. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 entering to 1 gone. |

```ogt
struct BannerResult {
  doc "Where a banner is."
  field visible bool "On screen."
  field done bool "It finished this tick."
  field progress f32 "0 entering to 1 gone." {
    unit ratio
  }
}
```

<sub>Module `ui` line 5690</sub>
