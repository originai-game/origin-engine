<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SoundDesc"></a>

### `SoundDesc` (desc struct)

PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `Float32Array` | — | — | — | — | The samples, -1 to 1 (clamped), interleaved by channel: frames * channels values; at least one frame. |
| `rate` | `number` | `48000` | 8000 … 192000 | `hz` | — | Samples per second per channel. |
| `channels` | `number` | `1` | 1 … 2 | — | — | Interleaved channels, 1 or 2. |

```ogt
struct SoundDesc {
  doc "PCM a game made itself (og.audio.createSound): a tone, a procedural effect, a recording it synthesised."
  desc true
  field samples Float32Array "The samples, -1 to 1 (clamped), interleaved by channel: frames * channels values; at least one frame."
  field rate u32 "Samples per second per channel." {
    default 48000
    min 8000
    max 192000
    unit hz
  }
  field channels u32 "Interleaved channels, 1 or 2." {
    default 1
    min 1
    max 2
  }
}
```

<sub>`api.ogt` line 13385</sub>
