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
| `pipelinesRequested` | `number` | `0` | — | — | — | Pipelines the renderer has asked to compile off the frame so far (on the web those asked of the browser asynchronously, natively those given to the compile workers): with pipelinesReady, a loading screen's count, such as 41/96. Only goes up; 0 without a renderer or where pipelines compile in the frame. |
| `pipelinesReady` | `number` | `0` | — | — | — | How many of them are compiled. Only goes up; equal to pipelinesRequested when none compiles. |

```ogt
struct LoadingStatus {
  doc "Where loading is."
  field screen LoadingScreen "Who draws the loading screen on this host (natively: project.ogt's choice, as the web page would)."
  field stage String "The last stage the game reported (empty: none)."
  field fraction f32 "Its fraction."
  field label String "Its label."
  field done bool "Whether the game called og.loading.done."
  field idle bool "As og.loading.idle answers."
  field pipelinesRequested u32 "Pipelines the renderer has asked to compile off the frame so far (on the web those asked of the browser asynchronously, natively those given to the compile workers): with pipelinesReady, a loading screen's count, such as 41/96. Only goes up; 0 without a renderer or where pipelines compile in the frame."
  field pipelinesReady u32 "How many of them are compiled. Only goes up; equal to pipelinesRequested when none compiles."
}
```

<sub>Module `loading` line 24</sub>
