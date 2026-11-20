<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeTimings"></a>

### `ProbeTimings` (struct)

The fixed timings head of state().

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `number` | `0` | — | `ms` | — | Time the last tick took. |
| `render` | `number` | `0` | — | `ms` | — | Time the last frame's render took; 0 without a renderer. |
| `frameMs` | `number` | `0` | — | `ms` | — | step + render. |
| `systems` | `readonly ProbeTiming[]` | `[]` | — | — | — | The last tick's systems in run order, each with its milliseconds. |
| `passes` | `readonly ProbeTiming[]` | `[]` | — | — | — | The last rendered frame's render passes in the render graph's order, each with its milliseconds of CPU encoding (empty without a renderer). |
| `gpuPasses` | `readonly ProbeTiming[]` | `[]` | — | — | — | The render passes of the newest frame the GPU timed (a frame or two behind `passes`), each with its own GPU milliseconds (passes run at once may sum past the frame); a pass it could not time is left out; empty without timestamp queries (RenderCapabilities.gpuTimestamps). |
| `triangles` | `number` | `0` | — | — | — | Triangles the last frame's main pass drew (after culling and level of detail); 0 without a renderer. |

```ogt
struct ProbeTimings {
  doc "The fixed timings head of state()."
  field step f32 "Time the last tick took." {
    unit ms
  }
  field render f32 "Time the last frame's render took; 0 without a renderer." {
    unit ms
  }
  field frameMs f32 "step + render." {
    unit ms
  }
  field systems List.ProbeTiming "The last tick's systems in run order, each with its milliseconds."
  field passes List.ProbeTiming "The last rendered frame's render passes in the render graph's order, each with its milliseconds of CPU encoding (empty without a renderer)."
  field gpuPasses List.ProbeTiming "The render passes of the newest frame the GPU timed (a frame or two behind `passes`), each with its own GPU milliseconds (passes run at once may sum past the frame); a pass it could not time is left out; empty without timestamp queries (RenderCapabilities.gpuTimestamps)."
  field triangles u32 "Triangles the last frame's main pass drew (after culling and level of detail); 0 without a renderer."
}
```

<sub>Module `probe` line 69</sub>
