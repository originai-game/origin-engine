<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ShakeDesc"></a>

### `ShakeDesc` (desc struct)

A shake of an element: three sideways swings decaying to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `amount` | `number` | `0` | ≥ 0 | `cssPx` | — | The first swing; 0 is the style's motion.shake, else 6. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it takes to come to rest; 0 is motion.duration.slow. |
| `sound` | `boolean` | `false` | — | — | — | Also play the style's error sound, as a refused press does. |

```ogt
struct ShakeDesc {
  doc "A shake of an element: three sideways swings decaying to rest."
  desc true
  field amount f32 "The first swing; 0 is the style's motion.shake, else 6." {
    unit cssPx
    min 0
  }
  field durationMs f32 "How long it takes to come to rest; 0 is motion.duration.slow." {
    unit ms
    min 0
  }
  field sound bool "Also play the style's error sound, as a refused press does."
}
```

<sub>Module `ui` line 1865</sub>
