<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BurstDesc"></a>

### `BurstDesc` (desc struct)

Rays and sparks thrown from an element.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | Rays; 0 is 8. |
| `color` | `Paint \| null` | `null` | — | — | — | Their ink; null is the primary. |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | How far they fly from the element's centre; 0 is 1.4 times its size. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long they fly; 0 is motion.duration.slow plus base. |
| `delayMs` | `number` | `0` | ≥ 0 | `ms` | — | Wait this long after the element first appears. |

```ogt
struct BurstDesc {
  doc "Rays and sparks thrown from an element."
  desc true
  field count u32 "Rays; 0 is 8."
  field color Option.Paint "Their ink; null is the primary."
  field radius f32 "How far they fly from the element's centre; 0 is 1.4 times its size." {
    unit cssPx
    min 0
  }
  field durationMs f32 "How long they fly; 0 is motion.duration.slow plus base." {
    unit ms
    min 0
  }
  field delayMs f32 "Wait this long after the element first appears." {
    unit ms
    min 0
  }
}
```

<sub>Module `ui` line 1484</sub>
