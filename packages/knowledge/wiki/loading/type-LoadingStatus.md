<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="type-LoadingStatus"></a>

### `LoadingStatus` (struct)

Where loading is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `screen` | `LoadingScreen` | `"engine"` | — | — | — | Who draws the loading screen on this host (natively: project.ogt's choice, as the web page would). |
| `stage` | `string` | `""` | — | — | — | The last stage the game reported (empty: none). |
| `fraction` | `number` | `0` | — | — | — | Its fraction. |
| `label` | `string` | `""` | — | — | — | Its label. |
| `done` | `boolean` | `false` | — | — | — | Whether the game called og.loading.done. |
| `idle` | `boolean` | `false` | — | — | — | As og.loading.idle answers. |

```ogt
struct LoadingStatus {
  doc "Where loading is."
  field screen LoadingScreen "Who draws the loading screen on this host (natively: project.ogt's choice, as the web page would)."
  field stage String "The last stage the game reported (empty: none)."
  field fraction f32 "Its fraction."
  field label String "Its label."
  field done bool "Whether the game called og.loading.done."
  field idle bool "As og.loading.idle answers."
}
```

<sub>Module `loading` line 24</sub>
