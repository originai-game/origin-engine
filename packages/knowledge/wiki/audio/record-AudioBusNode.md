<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-AudioBusNode"></a>

### `og.AudioBusNode` (component)

A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays into its `parent`: a built-in bus or a user bus of a lower index, so the graph has no cycles (any other parent is master). Its volume is multiplied down the chain to master, so a footsteps bus under sfx is turned down with sfx. The first node in query order for an index counts; a removed node's sources play on their `bus`.

The graph reaches the mixer as commands in the stream (only what changed each tick), so it is deterministic and in the world hash.

Token `og.AudioBusNode`, registered under the schema name `og.AudioBusNode`; its value type is `AudioBusNode`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `4` | 4 … 15 | — | — | The bus's index, 4 to 15. |
| `parent` | `number` | `0` | 0 … 15 | — | — | The bus it plays into: 0 master, 1 music, 2 sfx, 3 ui, or a user bus below `index`. |
| `volume` | `number` | `1` | 0 … 2 | — | tunable | The bus's gain. |

```ogt
component AudioBusNode {
  doc "A user bus of the bus graph: bus `index` (4 to 15) sums the sources whose AudioSource.userBus (or AudioEvent.userBus) names it, runs its og.AudioBusFx (the one whose userBus is the index), and plays into its `parent`: a built-in bus or a user bus of a lower index, so the graph has no cycles (any other parent is master). Its volume is multiplied down the chain to master, so a footsteps bus under sfx is turned down with sfx. The first node in query order for an index counts; a removed node's sources play on their `bus`."
  doc "The graph reaches the mixer as commands in the stream (only what changed each tick), so it is deterministic and in the world hash."
  field index u32 "The bus's index, 4 to 15." {
    default 4
    min 4
    max 15
  }
  field parent u32 "The bus it plays into: 0 master, 1 music, 2 sfx, 3 ui, or a user bus below `index`." {
    min 0
    max 15
  }
  field volume f32 "The bus's gain." {
    default 1
    min 0
    max 2
    tunable true
  }
}
```

<sub>`api.ogt` line 13194</sub>
