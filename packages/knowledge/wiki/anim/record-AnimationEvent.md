<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-AnimationEvent"></a>

### `og.AnimationEvent` (event)

A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers; each says where it came from (`state`, `layer`) and whether its clip weighed most of its layer (`dominant`), and the game decides which to act on.

See `animation §3`.

Token `og.AnimationEvent`, registered under the schema name `og.AnimationEvent`; its value type is `AnimationEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The skeleton entity. |
| `clip` | `ClipAsset` | `0` | — | — | — | The clip asset the marker belongs to. |
| `name` | `string` | `""` | — | — | — | The marker's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The marker's time in the clip. |
| `weight` | `number` | `0` | — | — | — | The clip's blend weight when it fired. |
| `state` | `number` | `0` | — | — | — | The graph state that played the clip: its index in its layer in the graph file, as og.AnimationGraph.state numbers the base layer's; 0 for an og.AnimationPlayer's clip. |
| `layer` | `number` | `0` | — | — | — | The graph layer that played it, in file order (0 is the base); 0 for a player's clip. |
| `dominant` | `boolean` | `false` | — | — | — | Its clip weighed most of its layer this tick (a player's clips count as one layer). One clip per layer is dominant each tick, whether or not it fired; a tie goes to the state entered, then to the later clip of a blend. See `animation §3`. |

```ogt
event AnimationEvent {
  doc "A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers; each says where it came from (`state`, `layer`) and whether its clip weighed most of its layer (`dominant`), and the game decides which to act on."
  guide "animation §3"
  field entity Entity "The skeleton entity."
  field clip ClipAsset "The clip asset the marker belongs to."
  field name String "The marker's name (at most 63 bytes)."
  field time f32 "The marker's time in the clip." {
    unit s
  }
  field weight f32 "The clip's blend weight when it fired."
  field state u32 "The graph state that played the clip: its index in its layer in the graph file, as og.AnimationGraph.state numbers the base layer's; 0 for an og.AnimationPlayer's clip."
  field layer u32 "The graph layer that played it, in file order (0 is the base); 0 for a player's clip."
  field dominant bool "Its clip weighed most of its layer this tick (a player's clips count as one layer). One clip per layer is dominant each tick, whether or not it fired; a tie goes to the state entered, then to the later clip of a blend." {
    guide "animation §3"
  }
}
```

<sub>Module `anim` line 556</sub>
