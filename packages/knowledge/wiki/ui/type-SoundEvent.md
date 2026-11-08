<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SoundEvent"></a>

### `SoundEvent` (struct)

A UI sound of the style.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | hover, press, open, close, success or error. |
| `playbackRate` | `number` | `1` | — | — | — | Its pitch (1: as recorded); a cascade step raises it 6 %. |

```ogt
struct SoundEvent {
  doc "A UI sound of the style."
  field name String "hover, press, open, close, success or error."
  field playbackRate f32 "Its pitch (1: as recorded); a cascade step raises it 6 %." {
    default 1
  }
}
```

<sub>Module `ui` line 4897</sub>
