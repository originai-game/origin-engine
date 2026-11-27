<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="type-LoadingStatus"></a>

### `LoadingStatus` (struct)

Where loading is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `screen` | `LoadingScreen` | `"engine"` | — | — | — | Who draws the loading screen on this host (natively: checked project startup's choice, as the web page would). |
| `stage` | `string` | `""` | — | — | — | The last stage the game reported (empty: none). |
| `fraction` | `number` | `0` | — | — | — | Its fraction. |
| `label` | `string` | `""` | — | — | — | Its label. |
| `done` | `boolean` | `false` | — | — | — | Whether the game called og.loading.done. |
| `idle` | `boolean` | `false` | — | — | — | As og.loading.idle answers. |
| `pipelinesRequested` | `number` | `0` | — | — | — | Pipelines requested for off-frame compilation: asynchronously from the browser on the web, from compile workers natively. With pipelinesReady, a loading screen's count (41/96). Only goes up; 0 without a renderer or when pipelines compile in-frame. |
| `pipelinesReady` | `number` | `0` | — | — | — | How many of them are compiled. Only goes up; equal to pipelinesRequested when none compiles. |

```ogt
struct LoadingStatus {
  doc "Where loading is."
  field screen LoadingScreen "Who draws the loading screen on this host (natively: checked project startup's choice, as the web page would)."
  field stage String "The last stage the game reported (empty: none)."
  field fraction f32 "Its fraction."
  field label String "Its label."
  field done bool "Whether the game called og.loading.done."
  field idle bool "As og.loading.idle answers."
  field pipelinesRequested u32 "Pipelines requested for off-frame compilation: asynchronously from the browser on the web, from compile workers natively. With pipelinesReady, a loading screen's count (41/96). Only goes up; 0 without a renderer or when pipelines compile in-frame."
  field pipelinesReady u32 "How many of them are compiled. Only goes up; equal to pipelinesRequested when none compiles."
}
```

<sub>Module `loading` line 24</sub>
