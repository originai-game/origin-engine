<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-VirtualCursorDesc"></a>

### `VirtualCursorDesc` (desc struct)

A pad's virtual cursor (docs/ui.md §2.1). The zero value moves at 900 units a second, takes a quarter second from a fine start to full speed, has a half-strength magnet and scrolls 1200 units a second.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speed` | `number` | `0` | ≥ 0 | `cssPx` | — | Units a second at full tilt; 0 is 900. |
| `accel` | `number` | `0` | — | `ms` | — | Milliseconds a held stick takes from a third of its speed to all of it, for fine aim first; 0 is 250, negative is none. |
| `magnet` | `number` | `0` | ≤ 1 | — | — | 0 to 1: how much it slows over a control and, once the stick rests, pulls to the control's centre; 0 takes the style's motion magnet (the design language's timing table: 0.5), negative is none. |
| `scroll` | `number` | `0` | — | `cssPx` | — | Units a second the right stick scrolls under it at full tilt; 0 is 1200, negative leaves the right stick to the game. |

```ogt
struct VirtualCursorDesc {
  doc "A pad's virtual cursor (docs/ui.md §2.1). The zero value moves at 900 units a second, takes a quarter second from a fine start to full speed, has a half-strength magnet and scrolls 1200 units a second."
  desc true
  field speed f32 "Units a second at full tilt; 0 is 900." {
    unit cssPx
    min 0
  }
  field accel f32 "Milliseconds a held stick takes from a third of its speed to all of it, for fine aim first; 0 is 250, negative is none." {
    unit ms
  }
  field magnet f32 "0 to 1: how much it slows over a control and, once the stick rests, pulls to the control's centre; 0 takes the style's motion magnet (the design language's timing table: 0.5), negative is none." {
    max 1
  }
  field scroll f32 "Units a second the right stick scrolls under it at full tilt; 0 is 1200, negative leaves the right stick to the game." {
    unit cssPx
  }
}
```

<sub>Module `ui` line 1401</sub>
