<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeState"></a>

### `ProbeState` (struct)

The probe's state(): a fixed head, then the game's own facts.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | Fixed steps since load. |
| `paused` | `boolean` | `false` | — | — | — | Whether the clock is held. |
| `seed` | `number` | `0` | — | — | — | The seed the world was assembled from. |
| `timings` | `ProbeTimings` | `{ step: 0, render: 0, frameMs: 0, systems: [], passes: [], gpuPasses: [], triangles: 0 }` | — | — | — | The last tick's timings. |
| `entities` | `readonly ProbeEntityRef[]` | `[]` | — | — | — | The named entities. |
| `facts` | `unknown` | `null` | — | — | — | What GameDesc.facts returned, or null. |

```ogt
struct ProbeState {
  doc "The probe's state(): a fixed head, then the game's own facts."
  field tick u32 "Fixed steps since load."
  field paused bool "Whether the clock is held."
  field seed u32 "The seed the world was assembled from."
  field timings ProbeTimings "The last tick's timings."
  field entities List.ProbeEntityRef "The named entities."
  field facts Json "What GameDesc.facts returned, or null."
}
```

<sub>`api.ogt` line 24095</sub>
