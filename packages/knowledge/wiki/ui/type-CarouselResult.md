<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CarouselResult"></a>

### `CarouselResult` (struct)

What a carousel answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | The current page after this tick: keep it. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |

```ogt
struct CarouselResult {
  doc "What a carousel answered this tick."
  field index u32 "The current page after this tick: keep it."
  field changed bool "It changed this tick."
}
```

<sub>Module `ui` line 2296</sub>
