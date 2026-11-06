<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedCamera"></a>

### `DescribedCamera` (struct)

The camera as drawn, with its shake.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | Where it is. |
| `target` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | What it looks at. |
| `up` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | Its up. |
| `projection` | `string` | `""` | — | — | — | `perspective` or `orthographic`. |
| `fov` | `number \| null` | `null` | — | `deg` | — | The vertical field of view (perspective). |
| `height` | `number \| null` | `null` | — | `m` | — | The height in view (orthographic). |
| `near` | `number` | `0` | — | `m` | — | The near plane. |
| `far` | `number` | `0` | — | `m` | — | The far plane. |

```ogt
struct DescribedCamera {
  doc "The camera as drawn, with its shake."
  field position Array.f32.3 "Where it is."
  field target Array.f32.3 "What it looks at."
  field up Array.f32.3 "Its up."
  field projection String "`perspective` or `orthographic`."
  field fov Option.f32 "The vertical field of view (perspective)." {
    unit deg
  }
  field height Option.f32 "The height in view (orthographic)." {
    unit m
  }
  field near f32 "The near plane." {
    unit m
  }
  field far f32 "The far plane." {
    unit m
  }
}
```

<sub>`api.ogt` line 24487</sub>
