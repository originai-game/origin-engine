<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="record-VfxEvent"></a>

### `og.VfxEvent` (event)

A composed effect reached one of its `event` lines (an `impact` line included, named `impact`): the frame a fireball lands, a slash connects, a spell resolves. Read in the tick after, like every event; a replay sends the same.

Token `og.VfxEvent`, registered under the schema name `og.VfxEvent`; its value type is `VfxEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Vfx. |
| `asset` | `VfxAsset` | `0` | — | — | — | The effect. |
| `name` | `string` | `""` | — | — | — | The event's name (at most 63 bytes). |
| `time` | `number` | `0` | — | `s` | — | Its time in the effect. |

```ogt
event VfxEvent {
  doc "A composed effect reached one of its `event` lines (an `impact` line included, named `impact`): the frame a fireball lands, a slash connects, a spell resolves. Read in the tick after, like every event; a replay sends the same."
  field entity Entity "The entity with the og.Vfx."
  field asset VfxAsset "The effect."
  field name String "The event's name (at most 63 bytes)."
  field time f32 "Its time in the effect." {
    unit s
  }
}
```

<sub>Module `vfx` line 37</sub>
