<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-FlipbookClipInfo"></a>

### `FlipbookClipInfo` (struct)

One clip of a loaded flipbook sheet.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `number` | `0` | — | — | — | Its index, what Flipbook.clip takes. |
| `name` | `string` | `""` | — | — | — | Its name in the sheet. |
| `frames` | `number` | `0` | — | — | — | How many frames. |
| `duration` | `number` | `0` | — | `s` | — | Seconds of one cycle (a ping-pong cycle goes there and back). |
| `mode` | `FlipMode` | `"loop"` | — | — | — | How it runs past its end. |
| `events` | `readonly string[]` | `[]` | — | — | — | Its events' names, in frame order. |

```ogt
struct FlipbookClipInfo {
  doc "One clip of a loaded flipbook sheet."
  field clip u32 "Its index, what Flipbook.clip takes."
  field name String "Its name in the sheet."
  field frames u32 "How many frames."
  field duration f32 "Seconds of one cycle (a ping-pong cycle goes there and back)." {
    unit s
  }
  field mode FlipMode "How it runs past its end."
  field events List.String "Its events' names, in frame order."
}
```

<sub>Module `anim` line 708</sub>
