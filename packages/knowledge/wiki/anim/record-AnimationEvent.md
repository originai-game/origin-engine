<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-AnimationEvent"></a>

### `og.AnimationEvent` (event)

A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.

Token `og.AnimationEvent`, registered under the schema name `og.AnimationEvent`; its value type is `AnimationEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The skeleton entity. |
| `clip` | `ClipAsset` | `0` | — | — | — | The clip asset the marker belongs to. |
| `name` | `string` | `""` | — | — | — | The marker's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | The marker's time in the clip. |
| `weight` | `number` | `0` | — | — | — | The clip's blend weight when it fired. |

```ogt
event AnimationEvent {
  doc "A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers."
  field entity Entity "The skeleton entity."
  field clip ClipAsset "The clip asset the marker belongs to."
  field name String "The marker's name (at most 63 bytes)."
  field time f32 "The marker's time in the clip." {
    unit s
  }
  field weight f32 "The clip's blend weight when it fired."
}
```

<sub>`api.ogt` line 12005</sub>
