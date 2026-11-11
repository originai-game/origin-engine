<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-SoundInfo"></a>

### `SoundInfo` (struct)

A loaded sound.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rate` | `number` | `0` | — | `hz` | — | Samples per second. |
| `channels` | `number` | `0` | — | — | — | Interleaved channels. |
| `frames` | `number` | `0` | — | — | — | Samples per channel. |
| `seconds` | `number` | `0` | — | `s` | — | Its length. |
| `chunks` | `number` | `0` | — | — | — | The chunks it was cooked in, the first included; 0 when it was cooked whole. A chunked sound is never resident whole: the engine reads its chunks from about half a second behind each playhead to four seconds ahead and drops the rest. See `loading §3`. |

```ogt
struct SoundInfo {
  doc "A loaded sound."
  field rate u32 "Samples per second." {
    unit hz
  }
  field channels u32 "Interleaved channels."
  field frames u32 "Samples per channel."
  field seconds f32 "Its length." {
    unit s
  }
  field chunks u32 "The chunks it was cooked in, the first included; 0 when it was cooked whole. A chunked sound is never resident whole: the engine reads its chunks from about half a second behind each playhead to four seconds ahead and drops the rest." {
    guide "loading §3"
  }
}
```

<sub>Module `load` line 108</sub>
