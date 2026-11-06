<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeMemory"></a>

### `ProbeMemory` (struct)

What the engine holds, in one shape on both hosts; a figure a host cannot read is null. A publish page, whose probe has no `memory`, answers the same from the engine Worker (`__runtime_probe__.memory()`, docs/web-export.md).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `host` | `string` | `""` | — | — | — | `web` or `native`. |
| `linear` | `number \| null` | `null` | — | `bytes` | — | The wasm linear memory's size, which the browser reserved for the engine and never takes back; null natively. |
| `resident` | `number \| null` | `null` | — | `bytes` | — | The process's resident memory (Linux /proc/self/status, macOS task_info); null on the web, where a page cannot read its process's. |
| `heap` | `ProbeHeap \| null` | `null` | — | — | — | The engine's allocator; null where the host counts nothing. |
| `world` | `ProbeWorldMemory` | `{ entities: 0 }` | — | — | — | The world. |
| `stream` | `ProbeStreamMemory \| null` | `null` | — | — | — | The streaming budget. |
| `audio` | `number \| null` | `null` | — | `bytes` | — | The sound bank's samples in memory (decoded sounds and the resident chunks of streamed ones); null without audio. |
| `gpu` | `GpuMemory \| null` | `null` | — | — | — | The GPU memory the renderer holds (og.render.display's gpu); null without a renderer. |
| `deviceLosses` | `number \| null` | `null` | — | — | — | How many times the GPU device was lost and the renderer made again (og.render.display's deviceLosses): field telemetry reports it beside the GPU memory. Null without a renderer. |

```ogt
struct ProbeMemory {
  doc "What the engine holds, in one shape on both hosts; a figure a host cannot read is null. A publish page, whose probe has no `memory`, answers the same from the engine Worker (`__runtime_probe__.memory()`, docs/web-export.md)."
  field host String "`web` or `native`."
  field linear Option.f64 "The wasm linear memory's size, which the browser reserved for the engine and never takes back; null natively." {
    unit bytes
  }
  field resident Option.f64 "The process's resident memory (Linux /proc/self/status, macOS task_info); null on the web, where a page cannot read its process's." {
    unit bytes
  }
  field heap Option.ProbeHeap "The engine's allocator; null where the host counts nothing."
  field world ProbeWorldMemory "The world."
  field stream Option.ProbeStreamMemory "The streaming budget."
  field audio Option.f64 "The sound bank's samples in memory (decoded sounds and the resident chunks of streamed ones); null without audio." {
    unit bytes
  }
  field gpu Option.GpuMemory "The GPU memory the renderer holds (og.render.display's gpu); null without a renderer."
  field deviceLosses Option.u32 "How many times the GPU device was lost and the renderer made again (og.render.display's deviceLosses): field telemetry reports it beside the GPU memory. Null without a renderer."
}
```

<sub>`api.ogt` line 24223</sub>
